// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::BlockIdFetcher;
use anyhow::{ensure, format_err, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use logger::prelude::*;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::{Accumulator, AccumulatorTreeStore, MerkleAccumulator};
use starcoin_crypto::HashValue;
use starcoin_types::block::{BlockIdAndNumber, BlockNumber};
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

#[derive(Clone)]
pub struct BlockAccumulatorSyncTask {
    start_number: BlockNumber,
    target: AccumulatorInfo,
    fetcher: Arc<dyn BlockIdFetcher>,
    batch_size: u64,
}

impl BlockAccumulatorSyncTask {
    pub fn new<F>(
        start_number: BlockNumber,
        target: AccumulatorInfo,
        fetcher: F,
        batch_size: u64,
    ) -> Result<Self>
    where
        F: BlockIdFetcher + 'static,
    {
        ensure!(
            target.num_leaves > start_number,
            "target block number should > start_number"
        );
        Ok(Self {
            start_number,
            target,
            fetcher: Arc::new(fetcher),
            batch_size,
        })
    }
}

impl TaskState for BlockAccumulatorSyncTask {
    type Item = HashValue;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let start = self.start_number;
            let target = self.target.num_leaves;
            let mut max_size = target
                .checked_sub(start)
                .ok_or_else(|| format_err!("target block number should > start_number"))?;
            if max_size > self.batch_size {
                max_size = self.batch_size;
            }
            debug!(
                "Accumulator sync task: start_number: {}, target_number: {}",
                start, target
            );
            self.fetcher
                .fetch_block_ids(None, start, false, max_size)
                .await
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        let next_start_number = self.start_number.saturating_add(self.batch_size as u64);
        if next_start_number >= self.target.num_leaves {
            None
        } else {
            Some(Self {
                start_number: next_start_number,
                target: self.target.clone(),
                fetcher: self.fetcher.clone(),
                batch_size: self.batch_size,
            })
        }
    }

    fn total_items(&self) -> Option<u64> {
        Some(self.target.num_leaves.saturating_sub(self.start_number))
    }
}

pub struct AccumulatorCollector {
    accumulator: MerkleAccumulator,
    ancestor: BlockIdAndNumber,
    target: AccumulatorInfo,
}

impl AccumulatorCollector {
    pub fn new(
        store: Arc<dyn AccumulatorTreeStore>,
        ancestor: BlockIdAndNumber,
        start: AccumulatorInfo,
        target: AccumulatorInfo,
    ) -> Self {
        let accumulator = MerkleAccumulator::new_with_info(start, store);
        Self {
            accumulator,
            ancestor,
            target,
        }
    }
}

impl TaskResultCollector<HashValue> for AccumulatorCollector {
    type Output = (BlockIdAndNumber, MerkleAccumulator);

    fn collect(&mut self, item: HashValue) -> Result<CollectorState> {
        self.accumulator.append(&[item])?;
        self.accumulator.flush()?;
        if self.accumulator.num_leaves() == self.target.num_leaves {
            Ok(CollectorState::Enough)
        } else {
            Ok(CollectorState::Need)
        }
    }

    fn finish(self) -> Result<Self::Output> {
        let info = self.accumulator.get_info();
        ensure!(
            info == self.target,
            "Target accumulator: {:?}, but got: {:?}",
            self.target,
            info
        );
        Ok((self.ancestor, self.accumulator))
    }
}
