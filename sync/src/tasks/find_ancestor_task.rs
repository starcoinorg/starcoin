// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::BlockIdFetcher;
use crate::verified_rpc_client::SyncTargetMismatch;
use anyhow::{format_err, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::PeerId;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_types::block::{BlockIdAndNumber, BlockNumber};
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

/// Deepest fork the ancestor search walks back through. Main produces a block about every
/// 5-10 s, so this is weeks of blocks; an honest node is never forked this deep, while a peer
/// answering with ids that never match used to walk us all the way to genesis.
pub const MAX_ANCESTOR_DEPTH: u64 = 1_000_000;

/// The batch doubles after every miss, up to this many ids per request, so a deep walk costs
/// hundreds of requests instead of one per 10 blocks. Well under the server's request cap.
const MAX_ANCESTOR_BATCH: u64 = 1000;

#[derive(Clone)]
pub struct FindAncestorTask {
    start_number: BlockNumber,
    lowest_number: BlockNumber,
    batch_size: u64,
    fetcher: Arc<dyn BlockIdFetcher>,
}

impl FindAncestorTask {
    pub fn new<F>(
        current_number: BlockNumber,
        target_block_number: BlockNumber,
        batch_size: u64,
        fetcher: F,
    ) -> Self
    where
        F: BlockIdFetcher + 'static,
    {
        let start_number = std::cmp::min(current_number, target_block_number);
        Self {
            start_number,
            lowest_number: start_number.saturating_sub(MAX_ANCESTOR_DEPTH),
            batch_size,
            fetcher: Arc::new(fetcher),
        }
    }
}

#[cfg(test)]
impl FindAncestorTask {
    pub(crate) fn start_number(&self) -> BlockNumber {
        self.start_number
    }

    pub(crate) fn batch_size(&self) -> u64 {
        self.batch_size
    }
}

impl TaskState for FindAncestorTask {
    type Item = BlockIdAndNumber;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let current_number = self.start_number;
            let block_ids = self
                .fetcher
                .fetch_block_ids(None, current_number, true, self.batch_size)
                .await?;
            let id_and_numbers = block_ids
                .into_iter()
                .enumerate()
                .map(|(idx, id)| BlockIdAndNumber {
                    id,
                    number: current_number.saturating_sub(idx as u64),
                })
                .collect();
            Ok(id_and_numbers)
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        if self.start_number <= self.lowest_number {
            return None;
        }
        let next_number = std::cmp::max(
            self.start_number.saturating_sub(self.batch_size),
            self.lowest_number,
        );
        Some(Self {
            start_number: next_number,
            lowest_number: self.lowest_number,
            batch_size: std::cmp::min(self.batch_size.saturating_mul(2), MAX_ANCESTOR_BATCH),
            fetcher: self.fetcher.clone(),
        })
    }
}

pub struct AncestorCollector {
    accumulator: Arc<MerkleAccumulator>,
    ancestor: Option<BlockIdAndNumber>,
    target_peers: Vec<PeerId>,
}

impl AncestorCollector {
    pub fn new(accumulator: Arc<MerkleAccumulator>, target_peers: Vec<PeerId>) -> Self {
        Self {
            accumulator,
            ancestor: None,
            target_peers,
        }
    }
}

impl TaskResultCollector<BlockIdAndNumber> for AncestorCollector {
    type Output = BlockIdAndNumber;

    fn collect(&mut self, item: BlockIdAndNumber) -> Result<CollectorState> {
        let block_id = self
            .accumulator
            .get_leaf(item.number)?
            .ok_or_else(|| format_err!("Can not find block id by number: {}", item.number))?;
        if self.ancestor.is_some() {
            return Ok(CollectorState::Enough);
        }
        if block_id == item.id {
            self.ancestor = Some(item);
            Ok(CollectorState::Enough)
        } else {
            Ok(CollectorState::Need)
        }
    }

    fn finish(mut self) -> Result<Self::Output> {
        // Every honest chain shares our genesis and forks far less than MAX_ANCESTOR_DEPTH deep,
        // so a miss means the target peers served ids that are not a real chain.
        self.ancestor.take().ok_or_else(|| {
            SyncTargetMismatch {
                peers: self.target_peers,
                msg: format!(
                    "no common ancestor within {} blocks of the sync start",
                    MAX_ANCESTOR_DEPTH
                ),
            }
            .into()
        })
    }
}
