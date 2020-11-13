// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::{
    BlockConnectedEvent, BlockConnectedEventHandle, BlockFetcher, BlockLocalStore, NoOpEventHandle,
};
use anyhow::{format_err, Result};
use chain::BlockChain;
use futures::future::BoxFuture;
use futures::FutureExt;
use logger::prelude::*;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain_api::ChainWriter;
use starcoin_types::block::{Block, BlockInfo, BlockNumber};
use starcoin_vm_types::on_chain_config::GlobalTimeOnChain;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

#[derive(Clone)]
pub struct BlockSyncTask {
    accumulator: Arc<MerkleAccumulator>,
    start_number: BlockNumber,
    fetcher: Arc<dyn BlockFetcher>,
    // if check_local_store is true, get block from local first.
    check_local_store: bool,
    local_store: Arc<dyn BlockLocalStore>,
    batch_size: u64,
}

impl BlockSyncTask {
    pub fn new<F, S>(
        accumulator: MerkleAccumulator,
        start_number: BlockNumber,
        fetcher: F,
        check_local_store: bool,
        local_store: S,
        batch_size: u64,
    ) -> Self
    where
        F: BlockFetcher + 'static,
        S: BlockLocalStore + 'static,
    {
        Self {
            accumulator: Arc::new(accumulator),
            start_number,
            fetcher: Arc::new(fetcher),
            check_local_store,
            local_store: Arc::new(local_store),
            batch_size,
        }
    }
}

impl TaskState for BlockSyncTask {
    type Item = (Block, Option<BlockInfo>);

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let block_ids =
                self.accumulator
                    .get_leaves(self.start_number, false, self.batch_size)?;
            if block_ids.is_empty() {
                return Ok(vec![]);
            }
            if self.check_local_store {
                let block_with_info = self.local_store.get_block_with_info(block_ids.clone())?;
                let (no_exist_block_ids, result_map) =
                    block_ids.clone().into_iter().zip(block_with_info).fold(
                        (vec![], HashMap::new()),
                        |(mut no_exist_block_ids, mut result_map), (block_id, block_with_info)| {
                            match block_with_info {
                                Some((block, block_info)) => {
                                    result_map.insert(block_id, (block, block_info));
                                }
                                None => {
                                    no_exist_block_ids.push(block_id);
                                }
                            }
                            (no_exist_block_ids, result_map)
                        },
                    );
                debug!(
                    "[sync] get_block_with_info from local store, ids: {}, found: {}",
                    block_ids.len(),
                    result_map.len()
                );
                let mut result_map = if no_exist_block_ids.is_empty() {
                    result_map
                } else {
                    self.fetcher
                        .fetch_block(no_exist_block_ids)
                        .await?
                        .into_iter()
                        .fold(result_map, |mut result_map, block| {
                            result_map.insert(block.id(), (block, None));
                            result_map
                        })
                };
                //ensure return block's order same as request block_id's order.
                let result: Result<Vec<(Block, Option<BlockInfo>)>> = block_ids
                    .iter()
                    .map(|block_id| {
                        result_map
                            .remove(block_id)
                            .ok_or_else(|| format_err!("Get block by id {:?} failed", block_id))
                    })
                    .collect();
                result
            } else {
                Ok(self
                    .fetcher
                    .fetch_block(block_ids)
                    .await?
                    .into_iter()
                    .map(|block| (block, None))
                    .collect())
            }
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
                check_local_store: self.check_local_store,
                local_store: self.local_store.clone(),
                batch_size: self.batch_size,
            })
        }
    }

    fn total_items(&self) -> Option<u64> {
        Some(self.accumulator.num_leaves() - self.start_number)
    }
}

pub struct BlockCollector {
    chain: BlockChain,
    event_handle: Box<dyn BlockConnectedEventHandle>,
}

impl BlockCollector {
    pub fn new(chain: BlockChain) -> Self {
        Self {
            chain,
            event_handle: Box::new(NoOpEventHandle),
        }
    }

    pub fn new_with_handle<H>(chain: BlockChain, event_handle: H) -> Self
    where
        H: BlockConnectedEventHandle + 'static,
    {
        Self {
            chain,
            event_handle: Box::new(event_handle),
        }
    }
}

impl TaskResultCollector<(Block, Option<BlockInfo>)> for BlockCollector {
    type Output = BlockChain;

    fn collect(
        mut self: Pin<&mut Self>,
        item: (Block, Option<BlockInfo>),
    ) -> Result<CollectorState> {
        let (block, block_info) = item;
        let block_id = block.id();
        let timestamp = block.header().timestamp;
        match block_info {
            Some(block_info) => {
                //If block_info exists, it means that this block was already executed and try connect in the previous sync, but the sync task was interrupted.
                //So, we just need to update chain and continue
                self.chain.update_chain_head_with_info(block, block_info)?;
            }
            None => {
                self.chain.apply(block.clone())?;
                self.chain
                    .time_service()
                    .adjust(GlobalTimeOnChain::new(timestamp));
                if let Err(e) = self.event_handle.handle(BlockConnectedEvent { block }) {
                    error!(
                        "Send BlockConnectedEvent error: {:?}, block_id: {}",
                        e, block_id
                    );
                }
            }
        }

        Ok(CollectorState::Need)
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(self.chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::format_err;
    use futures::FutureExt;
    use futures_timer::Delay;
    use starcoin_accumulator::accumulator_info::AccumulatorInfo;
    use starcoin_accumulator::tree_store::mock::MockAccumulatorStore;
    use starcoin_accumulator::MerkleAccumulator;
    use starcoin_crypto::HashValue;
    use starcoin_types::block::BlockHeader;
    use starcoin_types::U256;
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
        fn fetch_block(&self, block_ids: Vec<HashValue>) -> BoxFuture<Result<Vec<Block>>> {
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
        let accumulator = MerkleAccumulator::new_empty(store);
        for i in 0..total_blocks {
            let mut header = BlockHeader::random();
            header.number = i;
            let block = Block::new(header, vec![]);
            accumulator.append(&[block.id()]).unwrap();
            fetcher.put(block);
        }
        (fetcher, accumulator)
    }

    #[derive(Default)]
    struct MockLocalBlockStore {
        store: Mutex<HashMap<HashValue, (Block, Option<BlockInfo>)>>,
    }

    impl MockLocalBlockStore {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn mock(&self, block: &Block) {
            let block_id = block.id();
            let block_info = BlockInfo::new(
                block_id,
                AccumulatorInfo::new(HashValue::random(), vec![], 0, 0),
                U256::from(1),
                AccumulatorInfo::new(HashValue::random(), vec![], 0, 0),
            );
            self.store
                .lock()
                .unwrap()
                .insert(block.id(), (block.clone(), Some(block_info)));
        }
    }

    impl BlockLocalStore for MockLocalBlockStore {
        fn get_block_with_info(
            &self,
            block_ids: Vec<HashValue>,
        ) -> Result<Vec<Option<(Block, Option<BlockInfo>)>>> {
            let store = self.store.lock().unwrap();
            Ok(block_ids.iter().map(|id| store.get(id).cloned()).collect())
        }
    }

    #[stest::test]
    async fn test_block_sync() -> Result<()> {
        let total_blocks = 100;
        let (fetcher, accumulator) = build_block_fetcher(total_blocks);

        let block_sync_state = BlockSyncTask::new(
            accumulator,
            0,
            fetcher,
            false,
            MockLocalBlockStore::new(),
            3,
        );
        let event_handle = Arc::new(TaskEventCounterHandle::new());
        let sync_task =
            TaskGenerator::new(block_sync_state, 5, 3, 1, vec![], event_handle.clone()).generate();
        let result = sync_task.await?;
        let last_block_number = result
            .iter()
            .map(|(block, block_info)| {
                assert!(block_info.is_none());
                block.header().number as i64
            })
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

    #[stest::test]
    async fn test_block_sync_with_local() -> Result<()> {
        let total_blocks = 100;
        let (fetcher, accumulator) = build_block_fetcher(total_blocks);

        let local_store = MockLocalBlockStore::new();
        fetcher
            .blocks
            .lock()
            .unwrap()
            .iter()
            .for_each(|(_block_id, block)| {
                if block.header().number % 2 == 0 {
                    local_store.mock(block)
                }
            });
        let block_sync_state = BlockSyncTask::new(accumulator, 0, fetcher, true, local_store, 3);
        let event_handle = Arc::new(TaskEventCounterHandle::new());
        let sync_task =
            TaskGenerator::new(block_sync_state, 5, 3, 1, vec![], event_handle.clone()).generate();
        let result = sync_task.await?;
        let last_block_number = result
            .iter()
            .map(|(block, block_info)| {
                if block.header().number() % 2 == 0 {
                    assert!(block_info.is_some())
                } else {
                    assert!(block_info.is_none())
                }
                block.header().number as i64
            })
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
