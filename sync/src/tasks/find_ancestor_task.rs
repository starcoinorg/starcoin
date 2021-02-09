// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::BlockIdFetcher;
use anyhow::{format_err, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_types::block::{BlockIdAndNumber, BlockNumber};
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

#[derive(Clone)]
pub struct FindAncestorTask {
    start_number: BlockNumber,
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
        Self {
            start_number: std::cmp::min(current_number, target_block_number),
            batch_size,
            fetcher: Arc::new(fetcher),
        }
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
        //this should never happen, because all node's genesis block should same.
        if self.start_number == 0 {
            return None;
        }

        let next_number = self.start_number.saturating_sub(self.batch_size);
        Some(Self {
            start_number: next_number,
            batch_size: self.batch_size,
            fetcher: self.fetcher.clone(),
        })
    }
}

pub struct AncestorCollector {
    accumulator: Arc<MerkleAccumulator>,
    ancestor: Option<BlockIdAndNumber>,
}

impl AncestorCollector {
    pub fn new(accumulator: Arc<MerkleAccumulator>) -> Self {
        Self {
            accumulator,
            ancestor: None,
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
        self.ancestor
            .take()
            .ok_or_else(|| format_err!("Unexpect state, collector finished by ancestor is None"))
    }
}
