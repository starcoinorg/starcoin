// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::{format_err, Result};
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use logger::prelude::*;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::MerkleAccumulator;
use starcoin_crypto::HashValue;
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockHeader, BlockInfo, BlockNumber};
use std::sync::Arc;
use stream_task::{
    Generator, TaskError, TaskEventCounterHandle, TaskGenerator, TaskHandle, TaskResultCollector,
};

pub trait BlockIdFetcher: Send + Sync {
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

impl<T> BlockIdFetcher for Arc<T>
where
    T: BlockIdFetcher,
{
    fn fetch_block_ids(
        &self,
        start_number: u64,
        reverse: bool,
        max_size: usize,
    ) -> BoxFuture<'_, Result<Vec<HashValue>>> {
        BlockIdFetcher::fetch_block_ids(self.as_ref(), start_number, reverse, max_size)
    }
}

pub trait BlockFetcher: Send + Sync {
    fn fetch_block(&self, block_ids: Vec<HashValue>) -> BoxFuture<Result<Vec<Block>>>;
}

impl<T> BlockFetcher for Arc<T>
where
    T: BlockFetcher,
{
    fn fetch_block(&self, block_ids: Vec<HashValue>) -> BoxFuture<'_, Result<Vec<Block>>> {
        BlockFetcher::fetch_block(self.as_ref(), block_ids)
    }
}

impl BlockFetcher for VerifiedRpcClient {
    fn fetch_block(&self, block_ids: Vec<HashValue>) -> BoxFuture<'_, Result<Vec<Block>>> {
        self.get_blocks(block_ids.clone())
            .and_then(|blocks| async move {
                let results: Result<Vec<Block>> = block_ids
                    .iter()
                    .zip(blocks)
                    .map(|(id, block)| {
                        block.ok_or_else(|| {
                            format_err!("Get block by id: {} failed, remote node return None", id)
                        })
                    })
                    .collect();
                results
            })
            .boxed()
    }
}

pub trait BlockInfoFetcher: Send + Sync {
    fn fetch_block_infos(&self, block_ids: Vec<HashValue>) -> BoxFuture<Result<Vec<BlockInfo>>>;
}

impl<T> BlockInfoFetcher for Arc<T>
where
    T: BlockInfoFetcher,
{
    fn fetch_block_infos(
        &self,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<'_, Result<Vec<BlockInfo>>> {
        BlockInfoFetcher::fetch_block_infos(self.as_ref(), block_ids)
    }
}

#[derive(Debug, Clone)]
pub struct SyncTarget {
    pub block_header: BlockHeader,
    pub block_info: BlockInfo,
    pub peers: Vec<PeerInfo>,
}

mod accumulator_sync_task;
mod block_sync_task;
mod find_ancestor_task;
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

pub use accumulator_sync_task::{AccumulatorCollector, BlockAccumulatorSyncTask};
pub use block_sync_task::{BlockCollector, BlockSyncTask};
pub use find_ancestor_task::{AncestorCollector, FindAncestorTask};
use starcoin_types::peer_info::PeerInfo;

pub fn full_sync_task<C, F>(
    current_block_id: HashValue,
    target: BlockInfo,
    storage: Arc<dyn Store>,
    block_collector: C,
    fetcher: F,
) -> Result<(
    BoxFuture<'static, Result<C::Output, TaskError>>,
    TaskHandle,
    Arc<TaskEventCounterHandle>,
)>
where
    C: TaskResultCollector<Block> + 'static,
    F: BlockIdFetcher + BlockFetcher + 'static,
{
    let fetcher = Arc::new(fetcher);
    let current_block_header = storage
        .get_block_header_by_hash(current_block_id)?
        .ok_or_else(|| format_err!("Can not find block header by id: {}", current_block_id))?;
    let current_block_number = current_block_header.number;
    let current_block_id = current_block_header.id();
    let current_block_info = storage
        .get_block_info(current_block_id)?
        .ok_or_else(|| format_err!("Can not find block info by id: {}", current_block_id))?;

    let event_handle = Arc::new(TaskEventCounterHandle::new());

    let target_block_accumulator = target.block_accumulator_info;
    let current_block_accumulator_info = current_block_info.block_accumulator_info;

    let accumulator_task_fetcher = fetcher.clone();
    let block_task_fetcher = fetcher.clone();

    let max_retry_times = 15;
    let delay_milliseconds_on_error = 100;
    let sync_task = TaskGenerator::new(
        FindAncestorTask::new(current_block_number, 10, fetcher),
        3,
        max_retry_times,
        delay_milliseconds_on_error,
        AncestorCollector::new(Arc::new(MerkleAccumulator::new_with_info(
            current_block_accumulator_info,
            storage.get_accumulator_store(AccumulatorStoreType::Block),
        ))),
        event_handle.clone(),
    )
    .and_then(move |ancestor, event_handle| {
        debug!("find ancestor: {:?}", ancestor);
        let ancestor_block_info = storage.get_block_info(ancestor.id)?.ok_or_else(|| {
            format_err!("Can not find ancestor block info by id: {}", ancestor.id)
        })?;

        let accumulator_sync_task = BlockAccumulatorSyncTask::new(
            // start_number is include, so start from ancestor.number + 1
            ancestor.number + 1,
            target_block_accumulator.clone(),
            accumulator_task_fetcher,
            5,
        );
        Ok(TaskGenerator::new(
            accumulator_sync_task,
            3,
            max_retry_times,
            delay_milliseconds_on_error,
            AccumulatorCollector::new(
                storage.get_accumulator_store(AccumulatorStoreType::Block),
                ancestor_block_info.block_accumulator_info,
                target_block_accumulator,
            ),
            event_handle,
        ))
    })
    .and_then(move |accumulator, event_handle| {
        //start_number is include, so start from current_number + 1
        let block_sync_task =
            BlockSyncTask::new(accumulator, current_block_number + 1, block_task_fetcher, 3);
        Ok(TaskGenerator::new(
            block_sync_task,
            2,
            max_retry_times,
            delay_milliseconds_on_error,
            block_collector,
            event_handle,
        ))
    })
    .generate();
    let (fut, handle) = sync_task.with_handle();
    Ok((fut, handle, event_handle))
}
