// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::BlockIdFetcher;
use anyhow::{ensure, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use logger::prelude::*;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::{Accumulator, AccumulatorTreeStore, MerkleAccumulator};
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;
use std::pin::Pin;
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

#[derive(Clone)]
pub struct BlockAccumulatorSyncTask {
    start_number: BlockNumber,
    target: AccumulatorInfo,
    fetcher: Arc<dyn BlockIdFetcher>,
    batch_size: usize,
}

impl BlockAccumulatorSyncTask {
    pub fn new<F>(
        start_number: BlockNumber,
        target: AccumulatorInfo,
        fetcher: F,
        batch_size: usize,
    ) -> Self
    where
        F: BlockIdFetcher + 'static,
    {
        Self {
            start_number,
            target,
            fetcher: Arc::new(fetcher),
            batch_size,
        }
    }
}

impl TaskState for BlockAccumulatorSyncTask {
    type Item = HashValue;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let start = self.start_number;
            let target = self.target.num_leaves;
            let mut max_size = (target - start) as usize;
            if max_size > self.batch_size {
                max_size = self.batch_size;
            }
            debug!(
                "Accumulator sync task: start_number: {}, target_number: {}",
                start, target
            );
            self.fetcher.fetch_block_ids(start, false, max_size).await
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        let next_start_number = self.start_number + (self.batch_size as u64);
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
        Some(self.target.num_leaves - self.start_number)
    }
}

pub struct AccumulatorCollector {
    accumulator: MerkleAccumulator,
    target: AccumulatorInfo,
}

impl AccumulatorCollector {
    pub fn new(
        store: Arc<dyn AccumulatorTreeStore>,
        start: AccumulatorInfo,
        target: AccumulatorInfo,
    ) -> Self {
        let accumulator = MerkleAccumulator::new_with_info(start, store);
        Self {
            accumulator,
            target,
        }
    }
}

impl TaskResultCollector<HashValue> for AccumulatorCollector {
    type Output = MerkleAccumulator;

    fn collect(self: Pin<&mut Self>, item: HashValue) -> Result<CollectorState> {
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
        Ok(self.accumulator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::mock::MockBlockIdFetcher;
    use starcoin_accumulator::tree_store::mock::MockAccumulatorStore;
    use starcoin_accumulator::MerkleAccumulator;
    use stream_task::{Generator, TaskEventCounterHandle, TaskGenerator};

    #[stest::test]
    async fn test_accumulator_sync_by_stream_task() -> Result<()> {
        let store = Arc::new(MockAccumulatorStore::new());
        let accumulator = MerkleAccumulator::new_empty(store.clone());
        for _i in 0..100 {
            accumulator.append(&[HashValue::random()])?;
        }
        accumulator.flush().unwrap();
        let info0 = accumulator.get_info();
        assert_eq!(info0.num_leaves, 100);
        for _i in 0..100 {
            accumulator.append(&[HashValue::random()])?;
        }
        accumulator.flush().unwrap();
        let info1 = accumulator.get_info();
        assert_eq!(info1.num_leaves, 200);
        let fetcher = MockBlockIdFetcher::new(Arc::new(accumulator));
        let store2 = MockAccumulatorStore::copy_from(store.as_ref());

        let task_state = BlockAccumulatorSyncTask::new(info0.num_leaves, info1.clone(), fetcher, 7);
        let collector = AccumulatorCollector::new(Arc::new(store2), info0, info1.clone());
        let event_handle = Arc::new(TaskEventCounterHandle::new());
        let sync_task =
            TaskGenerator::new(task_state, 5, 3, 1, collector, event_handle.clone()).generate();
        let info2 = sync_task.await?.get_info();
        assert_eq!(info1, info2);
        let report = event_handle.get_reports().pop().unwrap();
        debug!("report: {}", report);
        Ok(())
    }
}
