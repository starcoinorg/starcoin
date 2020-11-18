// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::{format_err, Result};
use chain::BlockChain;
use futures::channel::mpsc::UnboundedSender;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use logger::prelude::*;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, EventHandler, ServiceRef};
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockHeader, BlockIdAndNumber, BlockInfo, BlockNumber};
use starcoin_types::peer_info::PeerInfo;
use starcoin_vm_types::time::TimeService;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use stream_task::{Generator, TaskError, TaskEventCounterHandle, TaskGenerator, TaskHandle};

pub trait BlockIdFetcher: Send + Sync {
    fn fetch_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>>;
}

impl BlockIdFetcher for VerifiedRpcClient {
    fn fetch_block_ids(
        &self,
        start_number: u64,
        reverse: bool,
        max_size: u64,
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
        max_size: u64,
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

pub trait BlockLocalStore: Send + Sync {
    fn get_block_with_info(
        &self,
        block_ids: Vec<HashValue>,
    ) -> Result<Vec<Option<(Block, Option<BlockInfo>)>>>;
}

// impl<T> BlockLocalStore for Arc<T>
// where
//     T: BlockLocalStore,
// {
//     fn get_block_with_info(
//         &self,
//         block_ids: Vec<HashValue>,
//     ) -> Result<Vec<Option<(Block, Option<BlockInfo>)>>> {
//         BlockLocalStore::get_block_with_info(self.as_ref(), block_ids)
//     }
// }

impl BlockLocalStore for Arc<dyn Store> {
    fn get_block_with_info(
        &self,
        block_ids: Vec<HashValue>,
    ) -> Result<Vec<Option<(Block, Option<BlockInfo>)>>> {
        self.get_blocks(block_ids)?
            .into_iter()
            .map(|block| match block {
                Some(block) => {
                    let id = block.id();
                    let block_info = self.get_block_info(id)?;
                    Ok(Some((block, block_info)))
                }
                None => Ok(None),
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct SyncTarget {
    pub block_header: BlockHeader,
    pub block_info: BlockInfo,
    pub peers: Vec<PeerInfo>,
}

#[derive(Clone, Debug)]
pub struct BlockConnectedEvent {
    pub block: Block,
}

pub trait BlockConnectedEventHandle: Send {
    fn handle(&mut self, event: BlockConnectedEvent) -> Result<()>;
}

impl<S> BlockConnectedEventHandle for ServiceRef<S>
where
    S: ActorService + EventHandler<S, BlockConnectedEvent>,
{
    fn handle(&mut self, event: BlockConnectedEvent) -> Result<()> {
        self.notify(event)?;
        Ok(())
    }
}

pub struct NoOpEventHandle;

impl BlockConnectedEventHandle for NoOpEventHandle {
    fn handle(&mut self, event: BlockConnectedEvent) -> Result<()> {
        debug!("Handle BlockConnectedEvent {:?}", event);
        Ok(())
    }
}

impl BlockConnectedEventHandle for Sender<BlockConnectedEvent> {
    fn handle(&mut self, event: BlockConnectedEvent) -> Result<()> {
        self.send(event)?;
        Ok(())
    }
}

impl BlockConnectedEventHandle for UnboundedSender<BlockConnectedEvent> {
    fn handle(&mut self, event: BlockConnectedEvent) -> Result<()> {
        self.start_send(event)?;
        Ok(())
    }
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

pub fn full_sync_task<H, F>(
    current_block_id: HashValue,
    target: BlockInfo,
    time_service: Arc<dyn TimeService>,
    storage: Arc<dyn Store>,
    block_event_handle: H,
    fetcher: F,
) -> Result<(
    BoxFuture<'static, Result<BlockChain, TaskError>>,
    TaskHandle,
    Arc<TaskEventCounterHandle>,
)>
where
    H: BlockConnectedEventHandle + 'static,
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
    let current_block_id_number = BlockIdAndNumber::new(current_block_id, current_block_number);

    let event_handle = Arc::new(TaskEventCounterHandle::new());

    let target_block_number = target.block_accumulator_info.num_leaves - 1;
    let target_block_accumulator = target.block_accumulator_info;

    let current_block_accumulator_info = current_block_info.block_accumulator_info;

    let accumulator_task_fetcher = fetcher.clone();
    let block_task_fetcher = fetcher.clone();
    let chain_storage = storage.clone();
    let max_retry_times = 15;
    let delay_milliseconds_on_error = 100;
    let sync_task =
        TaskGenerator::new(
            FindAncestorTask::new(current_block_number, target_block_number, 10, fetcher),
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
            info!("[sync] Find ancestor: {:?}", ancestor);
            let ancestor_block_info = storage.get_block_info(ancestor.id)?.ok_or_else(|| {
                format_err!(
                    "[sync] Can not find ancestor block info by id: {}",
                    ancestor.id
                )
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
                5,
                max_retry_times,
                delay_milliseconds_on_error,
                AccumulatorCollector::new(
                    storage.get_accumulator_store(AccumulatorStoreType::Block),
                    ancestor,
                    ancestor_block_info.block_accumulator_info,
                    target_block_accumulator,
                ),
                event_handle,
            ))
        })
        .and_then(move |(ancestor, accumulator), event_handle| {
            //start_number is include, so start from ancestor.number + 1
            let start_number = ancestor.number + 1;
            // if current block == ancestor, it means target is at current main chain's future, so do not check local store.
            // otherwise, we need to check if the block has already been executed to avoid wasting the results of previously interrupted task execution.
            let check_local_store = ancestor != current_block_id_number;
            info!(
            "[sync] Start sync block, ancestor: {:?}, start_number: {}, check_local_store: {:?}, target_number: {}", 
            ancestor, start_number, check_local_store, accumulator.num_leaves() -1 );
            let block_sync_task = BlockSyncTask::new(
                accumulator,
                start_number,
                block_task_fetcher,
                check_local_store,
                chain_storage.clone(),
                1,
            );
            let chain = BlockChain::new(time_service, ancestor.id, chain_storage)?;
            let block_collector = BlockCollector::new_with_handle(chain, block_event_handle);
            Ok(TaskGenerator::new(
                block_sync_task,
                5,
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
