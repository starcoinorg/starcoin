// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::{format_err, Result};
use futures::future::{BoxFuture, Future};
use futures::task::{Context, Poll};
use futures::FutureExt;
use logger::prelude::*;
use pin_utils::pin_mut;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::{Accumulator, AccumulatorTreeStore, MerkleAccumulator};
use starcoin_crypto::HashValue;
use starcoin_types::block::{Block, BlockNumber};
use std::pin::Pin;
use std::sync::Arc;

pub trait BlockIdFetcher: Send {
    fn fetch_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: usize,
    ) -> BoxFuture<Result<Vec<HashValue>>>;
}

impl BlockIdFetcher for VerifiedRpcClient {
    fn fetch_block_ids(
        &self,
        start_number: u64,
        reverse: bool,
        max_size: usize,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        self.get_block_ids(start_number, reverse, max_size).boxed()
    }
}

pub trait BlockFetcher {
    fn fetch(&self, block_id: HashValue) -> BoxFuture<Result<Block>>;
}

pub struct BlockAccumulatorSyncTask {
    accumulator: MerkleAccumulator,
    target: AccumulatorInfo,
    fetcher: Box<dyn BlockIdFetcher>,
    batch_size: usize,
}

impl BlockAccumulatorSyncTask {
    pub fn new<F>(
        store: Arc<dyn AccumulatorTreeStore>,
        current: AccumulatorInfo,
        target: AccumulatorInfo,
        fetcher: F,
        batch_size: usize,
    ) -> Self
    where
        F: BlockIdFetcher + 'static,
    {
        let accumulator = MerkleAccumulator::new_with_info(current, store);
        Self {
            accumulator,
            target,
            fetcher: Box::new(fetcher),
            batch_size,
        }
    }
}

impl Future for BlockAccumulatorSyncTask {
    type Output = Result<AccumulatorInfo>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let start = self.accumulator.num_leaves();
            let target = self.target.num_leaves;
            debug!(
                "Accumulator sync task: start_number: {}, target_number: {}",
                start, target
            );
            if start == target {
                let current_info = self.accumulator.get_info();
                return if self.target == current_info {
                    Poll::Ready(Ok(current_info))
                } else {
                    Poll::Ready(Err(format_err!(
                        "Verify accumulator root fail, expect: {:?}, but get: {:?}",
                        target,
                        current_info
                    )))
                };
            } else if start >= target {
                unreachable!("BlockAccumulatorSyncTask start > target")
            } else {
                let mut max_size = (target - start) as usize;
                if max_size > self.batch_size {
                    max_size = self.batch_size;
                }
                let block_ids_fut = self.fetcher.fetch_block_ids(start, false, max_size);
                pin_mut!(block_ids_fut);
                match block_ids_fut.poll(cx) {
                    Poll::Ready(result) => match result {
                        Err(e) => {
                            //TODO add retry limit.
                            error!("Fetch block ids error: {:?}", e);
                            continue;
                        }
                        Ok(block_ids) => match self.accumulator.append(block_ids.as_slice()) {
                            Ok(_) => continue,
                            Err(e) => return Poll::Ready(Err(e)),
                        },
                    },
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;
    use starcoin_accumulator::tree_store::mock::MockAccumulatorStore;
    use starcoin_accumulator::MerkleAccumulator;

    struct MockBlockIdFetcher {
        accumulator: MerkleAccumulator,
    }

    impl BlockIdFetcher for MockBlockIdFetcher {
        fn fetch_block_ids(
            &self,
            start_number: u64,
            reverse: bool,
            max_size: usize,
        ) -> BoxFuture<Result<Vec<HashValue>>> {
            let ids = self.accumulator.get_leaves(start_number, reverse, max_size);
            async move { ids }.boxed()
        }
    }

    #[stest::test]
    async fn test_accumulator_sync() -> Result<()> {
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
        for i in 0..200 {
            accumulator.get_leaf(i).unwrap().unwrap();
        }
        let fetcher = MockBlockIdFetcher { accumulator };
        let store2 = MockAccumulatorStore::copy_from(store.as_ref());
        let sync_task =
            BlockAccumulatorSyncTask::new(Arc::new(store2), info0, info1.clone(), fetcher, 7);
        let info2 = sync_task.await?;
        assert_eq!(info1, info2);
        Ok(())
    }
    
}
