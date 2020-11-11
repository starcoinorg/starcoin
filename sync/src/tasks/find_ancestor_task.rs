// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_sync::BlockIdAndNumber;
use crate::tasks::BlockIdFetcher;
use anyhow::{format_err, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_types::block::BlockNumber;
use std::pin::Pin;
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
                .fetch_block_ids(current_number, true, self.batch_size as usize)
                .await?;
            let id_and_numbers = block_ids
                .into_iter()
                .enumerate()
                .map(|(idx, id)| BlockIdAndNumber {
                    id,
                    number: current_number - (idx as u64),
                })
                .collect();
            Ok(id_and_numbers)
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        let next_number = self.start_number.saturating_sub(self.batch_size);

        //this should never happen, because all node's genesis block should same.
        if next_number == 0 {
            return None;
        }
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

    fn collect(mut self: Pin<&mut Self>, item: BlockIdAndNumber) -> Result<CollectorState> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::mock::MockBlockIdFetcher;
    use logger::prelude::*;
    use starcoin_accumulator::tree_store::mock::MockAccumulatorStore;
    use starcoin_accumulator::{Accumulator, MerkleAccumulator};
    use starcoin_crypto::HashValue;
    use std::sync::Arc;
    use stream_task::{Generator, TaskEventCounterHandle, TaskGenerator};

    #[stest::test]
    pub async fn test_find_ancestor_same_number() -> Result<()> {
        let store = Arc::new(MockAccumulatorStore::new());
        let accumulator = Arc::new(MerkleAccumulator::new_empty(store.clone()));

        let fetcher = MockBlockIdFetcher::new(accumulator.clone());
        fetcher.appends(generate_hash(100).as_slice())?;
        let info0 = accumulator.get_info();

        let store2 = Arc::new(MockAccumulatorStore::copy_from(store.as_ref()));
        let accumulator2 = Arc::new(MerkleAccumulator::new_with_info(info0.clone(), store2));
        let task_state = FindAncestorTask::new(
            accumulator2.num_leaves() - 1,
            accumulator.num_leaves() - 1,
            7,
            fetcher.clone(),
        );
        let event_handle = Arc::new(TaskEventCounterHandle::new());
        let collector = AncestorCollector::new(accumulator2.clone());
        let task =
            TaskGenerator::new(task_state, 5, 3, 1, collector, event_handle.clone()).generate();
        let ancestor = task.await?;
        assert_eq!(ancestor.number, info0.num_leaves - 1);
        let report = event_handle.get_reports().pop().unwrap();
        debug!("report: {}", report);

        Ok(())
    }

    #[stest::test]
    pub async fn test_find_ancestor_block_number_behind() -> Result<()> {
        let store = Arc::new(MockAccumulatorStore::new());
        let accumulator = Arc::new(MerkleAccumulator::new_empty(store.clone()));

        let fetcher = MockBlockIdFetcher::new(accumulator.clone());
        fetcher.appends(generate_hash(100).as_slice())?;
        let info0 = accumulator.get_info();

        // remote node block id is greater than local.
        fetcher.appends(generate_hash(100).as_slice())?;

        let store2 = Arc::new(MockAccumulatorStore::copy_from(store.as_ref()));
        let accumulator2 = Arc::new(MerkleAccumulator::new_with_info(info0.clone(), store2));
        let task_state = FindAncestorTask::new(
            accumulator2.num_leaves() - 1,
            accumulator.num_leaves() - 1,
            7,
            fetcher.clone(),
        );
        let event_handle = Arc::new(TaskEventCounterHandle::new());
        let collector = AncestorCollector::new(accumulator2.clone());
        let task =
            TaskGenerator::new(task_state, 5, 3, 1, collector, event_handle.clone()).generate();
        let ancestor = task.await?;
        assert_eq!(ancestor.number, info0.num_leaves - 1);
        let report = event_handle.get_reports().pop().unwrap();
        debug!("report: {}", report);

        Ok(())
    }

    fn generate_hash(count: usize) -> Vec<HashValue> {
        (0..count).map(|_| HashValue::random()).collect::<Vec<_>>()
    }

    #[stest::test]
    pub async fn test_find_ancestor_chain_fork() -> Result<()> {
        let store = Arc::new(MockAccumulatorStore::new());
        let accumulator = Arc::new(MerkleAccumulator::new_empty(store.clone()));

        let fetcher = MockBlockIdFetcher::new(accumulator.clone());
        fetcher.appends(generate_hash(100).as_slice())?;
        let info0 = accumulator.get_info();

        fetcher.appends(generate_hash(100).as_slice())?;

        let store2 = Arc::new(MockAccumulatorStore::copy_from(store.as_ref()));
        let accumulator2 = Arc::new(MerkleAccumulator::new_with_info(info0.clone(), store2));

        accumulator2.append(generate_hash(100).as_slice())?;
        accumulator2.flush()?;

        assert_ne!(accumulator.get_info(), accumulator2.get_info());

        let task_state = FindAncestorTask::new(
            accumulator2.num_leaves() - 1,
            accumulator.num_leaves() - 1,
            7,
            fetcher.clone(),
        );
        let event_handle = Arc::new(TaskEventCounterHandle::new());
        let collector = AncestorCollector::new(accumulator2.clone());
        let task =
            TaskGenerator::new(task_state, 5, 3, 1, collector, event_handle.clone()).generate();
        let ancestor = task.await?;
        assert_eq!(ancestor.number, info0.num_leaves - 1);
        let report = event_handle.get_reports().pop().unwrap();
        debug!("report: {}", report);
        assert_eq!(report.processed_items, 100 + 1);
        Ok(())
    }
}
