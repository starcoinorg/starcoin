// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::BlockFetcher;
use anyhow::Result;
use futures::future::BoxFuture;
use futures::FutureExt;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_types::block::{Block, BlockNumber};
use std::sync::Arc;
use stream_task::TaskState;

#[derive(Clone)]
pub struct BlockSyncTask {
    accumulator: Arc<MerkleAccumulator>,
    start_number: BlockNumber,
    fetcher: Arc<dyn BlockFetcher>,
    batch_size: u64,
}

impl BlockSyncTask {
    pub fn new<F>(
        accumulator: MerkleAccumulator,
        start_number: BlockNumber,
        fetcher: F,
        batch_size: u64,
    ) -> Self
    where
        F: BlockFetcher + 'static,
    {
        Self {
            accumulator: Arc::new(accumulator),
            start_number,
            fetcher: Arc::new(fetcher),
            batch_size,
        }
    }
}

impl TaskState for BlockSyncTask {
    type Item = Block;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let block_ids =
                self.accumulator
                    .get_leaves(self.start_number, false, self.batch_size as usize)?;
            if block_ids.is_empty() {
                return Ok(vec![]);
            }
            self.fetcher.fetch(block_ids).await
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        let next_start_number = self.start_number + self.batch_size;
        if next_start_number > self.accumulator.num_leaves() {
            None
        } else {
            Some(Self {
                accumulator: self.accumulator.clone(),
                start_number: next_start_number,
                fetcher: self.fetcher.clone(),
                batch_size: self.batch_size,
            })
        }
    }

    fn total_items(&self) -> Option<u64> {
        Some(self.accumulator.num_leaves() - self.start_number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::format_err;
    use futures::FutureExt;
    use futures_timer::Delay;
    use logger::prelude::*;
    use starcoin_accumulator::tree_store::mock::MockAccumulatorStore;
    use starcoin_accumulator::MerkleAccumulator;
    use starcoin_crypto::HashValue;
    use starcoin_types::block::BlockHeader;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::Duration;
    use stream_task::{Generator, TaskEventCounterHandle, TaskGenerator};

    #[derive(Default)]
    struct MockBlockFetcher {
        blocks: Mutex<HashMap<HashValue, Block>>,
    }

    impl MockBlockFetcher {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn put(&self, block: Block) {
            self.blocks.lock().unwrap().insert(block.id(), block);
        }
    }

    impl BlockFetcher for MockBlockFetcher {
        fn fetch(&self, block_ids: Vec<HashValue>) -> BoxFuture<Result<Vec<Block>>> {
            let blocks = self.blocks.lock().unwrap();
            let result: Result<Vec<Block>> = block_ids
                .iter()
                .map(|block_id| {
                    blocks
                        .get(block_id)
                        .cloned()
                        .ok_or_else(|| format_err!("Can not find block by id: {:?}", block_id))
                })
                .collect();
            async {
                Delay::new(Duration::from_millis(100)).await;
                result
            }
            .boxed()
        }
    }

    fn build_block_fetcher(total_blocks: u64) -> (MockBlockFetcher, MerkleAccumulator) {
        let fetcher = MockBlockFetcher::new();

        let store = Arc::new(MockAccumulatorStore::new());
        let accumulator = MerkleAccumulator::new_empty(store.clone());
        for i in 0..total_blocks {
            let mut header = BlockHeader::random();
            header.number = i;
            let block = Block::new(header, vec![]);
            accumulator.append(&[block.id()]).unwrap();
            fetcher.put(block);
        }
        (fetcher, accumulator)
    }

    #[stest::test]
    async fn test_block_sync() -> Result<()> {
        let total_blocks = 100;
        let (fetcher, accumulator) = build_block_fetcher(total_blocks);
        let block_sync_state = BlockSyncTask::new(accumulator, 0, fetcher, 3);
        let event_handle = Arc::new(TaskEventCounterHandle::new());
        let sync_task =
            TaskGenerator::new(block_sync_state, 5, 3, 1, vec![], event_handle.clone()).generate();
        let result = sync_task.await?;
        let last_block_number = result
            .iter()
            .map(|block| block.header().number as i64)
            .fold(-1, |parent, current| {
                //ensure return block is ordered
                assert_eq!(
                    parent + 1,
                    current,
                    "block sync task not return ordered blocks"
                );
                current
            });

        assert_eq!(last_block_number as u64, total_blocks - 1);

        let report = event_handle.get_reports().pop().unwrap();
        debug!("report: {}", report);
        Ok(())
    }
}
