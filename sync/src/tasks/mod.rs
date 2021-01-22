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
use starcoin_accumulator::MerkleAccumulator;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, EventHandler, ServiceRef};
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockIdAndNumber, BlockInfo, BlockNumber};
use starcoin_vm_types::time::TimeService;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use stream_task::{
    Generator, TaskError, TaskEventCounterHandle, TaskFuture, TaskGenerator, TaskHandle,
};

pub trait PeerOperator: Send + Sync {
    fn filter(&self, peers: &[PeerId]);

    fn new_peer(&self, beer_info: PeerInfo);

    fn peers(&self) -> Vec<PeerId>;
}

pub trait BlockIdFetcher: Send + Sync {
    fn fetch_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>>;

    fn fetch_block_ids_from_peer(
        &self,
        peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>>;

    fn fetch_block_infos_from_peer(
        &self,
        peer_id: Option<PeerId>,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>>;

    fn find_best_peer(&self) -> Option<PeerInfo>;
}

impl PeerOperator for VerifiedRpcClient {
    fn filter(&self, peers: &[PeerId]) {
        self.selector().retain(peers)
    }

    fn new_peer(&self, peer_info: PeerInfo) {
        self.selector().add_peer(peer_info);
    }

    fn peers(&self) -> Vec<PeerId> {
        self.selector()
            .peers()
            .into_iter()
            .map(|peer_info| peer_info.peer_id())
            .collect()
    }
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

    fn fetch_block_ids_from_peer(
        &self,
        peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        self.get_block_ids_from_peer(peer, start_number, reverse, max_size)
            .boxed()
    }

    fn fetch_block_infos_from_peer(
        &self,
        peer_id: Option<PeerId>,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>> {
        self.get_block_infos_from_peer(peer_id, hashes).boxed()
    }

    fn find_best_peer(&self) -> Option<PeerInfo> {
        self.best_peer()
    }
}

impl<T> PeerOperator for Arc<T>
where
    T: PeerOperator,
{
    fn filter(&self, peers: &[PeerId]) {
        PeerOperator::filter(self.as_ref(), peers)
    }

    fn new_peer(&self, peer_info: PeerInfo) {
        PeerOperator::new_peer(self.as_ref(), peer_info)
    }

    fn peers(&self) -> Vec<PeerId> {
        PeerOperator::peers(self.as_ref())
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

    fn fetch_block_ids_from_peer(
        &self,
        peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        BlockIdFetcher::fetch_block_ids_from_peer(
            self.as_ref(),
            peer,
            start_number,
            reverse,
            max_size,
        )
    }

    fn fetch_block_infos_from_peer(
        &self,
        peer_id: Option<PeerId>,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>> {
        BlockIdFetcher::fetch_block_infos_from_peer(self.as_ref(), peer_id, hashes)
    }

    fn find_best_peer(&self) -> Option<PeerInfo> {
        BlockIdFetcher::find_best_peer(self.as_ref())
    }
}

pub trait BlockFetcher: Send + Sync {
    fn fetch_block(
        &self,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<(Block, Option<PeerId>)>>>;
}

impl<T> BlockFetcher for Arc<T>
where
    T: BlockFetcher,
{
    fn fetch_block(
        &self,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<'_, Result<Vec<(Block, Option<PeerId>)>>> {
        BlockFetcher::fetch_block(self.as_ref(), block_ids)
    }
}

impl BlockFetcher for VerifiedRpcClient {
    fn fetch_block(
        &self,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<'_, Result<Vec<(Block, Option<PeerId>)>>> {
        self.get_blocks(block_ids.clone())
            .and_then(|blocks| async move {
                let results: Result<Vec<(Block, Option<PeerId>)>> = block_ids
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
    fn get_block_with_info(&self, block_ids: Vec<HashValue>) -> Result<Vec<Option<SyncBlockData>>>;
}

impl BlockLocalStore for Arc<dyn Store> {
    fn get_block_with_info(&self, block_ids: Vec<HashValue>) -> Result<Vec<Option<SyncBlockData>>> {
        self.get_blocks(block_ids)?
            .into_iter()
            .map(|block| match block {
                Some(block) => {
                    let id = block.id();
                    let block_info = self.get_block_info(id)?;
                    Ok(Some(SyncBlockData::new(block, block_info, None)))
                }
                None => Ok(None),
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct BlockConnectedEvent {
    pub block: Block,
}

pub trait BlockConnectedEventHandle: Send + Clone + std::marker::Unpin {
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

#[derive(Clone, Debug)]
pub struct AncestorEvent {
    pub ancestor: BlockIdAndNumber,
}

pub trait AncestorEventHandle: Send + Clone + std::marker::Unpin {
    fn handle(&mut self, event: AncestorEvent) -> Result<()>;
}

impl AncestorEventHandle for Sender<AncestorEvent> {
    fn handle(&mut self, event: AncestorEvent) -> Result<()> {
        self.send(event)?;
        Ok(())
    }
}

impl AncestorEventHandle for UnboundedSender<AncestorEvent> {
    fn handle(&mut self, event: AncestorEvent) -> Result<()> {
        self.start_send(event)?;
        Ok(())
    }
}

impl<S> AncestorEventHandle for ServiceRef<S>
where
    S: ActorService + EventHandler<S, AncestorEvent>,
{
    fn handle(&mut self, event: AncestorEvent) -> Result<()> {
        self.notify(event)?;
        Ok(())
    }
}

#[derive(Clone)]
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
mod inner_sync_task;
#[cfg(test)]
pub(crate) mod mock;
pub mod sync_score_metrics;
#[cfg(test)]
mod tests;

use crate::peer_event_handle::PeerEventHandle;
use crate::tasks::block_sync_task::SyncBlockData;
use crate::tasks::inner_sync_task::{FindSubTargetTask, InnerSyncTask};
pub use accumulator_sync_task::{AccumulatorCollector, BlockAccumulatorSyncTask};
pub use block_sync_task::{BlockCollector, BlockSyncTask};
pub use find_ancestor_task::{AncestorCollector, FindAncestorTask};
use futures::channel::mpsc::unbounded;
use network::get_unix_ts_as_millis;
use network_api::messages::PeerEvent;
use network_api::{NetworkService, PeerStrategy};
use starcoin_types::peer_info::{PeerId, PeerInfo};
use traits::ChainReader;

pub fn full_sync_task<H, A, F, N>(
    current_block_id: HashValue,
    target: BlockInfo,
    skip_pow_verify: bool,
    time_service: Arc<dyn TimeService>,
    storage: Arc<dyn Store>,
    block_event_handle: H,
    fetcher: Arc<F>,
    ancestor_event_handle: A,
    network: N,
    max_retry_times: u64,
    strategy: PeerStrategy,
) -> Result<(
    BoxFuture<'static, Result<BlockChain, TaskError>>,
    TaskHandle,
    Arc<TaskEventCounterHandle>,
    PeerEventHandle,
)>
where
    H: BlockConnectedEventHandle + Sync + 'static,
    A: AncestorEventHandle + Sync + 'static,
    F: BlockIdFetcher + BlockFetcher + PeerOperator + 'static,
    N: NetworkService + 'static,
{
    let current_block_header = storage
        .get_block_header_by_hash(current_block_id)?
        .ok_or_else(|| format_err!("Can not find block header by id: {}", current_block_id))?;
    let current_block_number = current_block_header.number();
    let current_block_id = current_block_header.id();
    let current_block_info = storage
        .get_block_info(current_block_id)?
        .ok_or_else(|| format_err!("Can not find block info by id: {}", current_block_id))?;

    let event_handle = Arc::new(TaskEventCounterHandle::new());

    let target_block_number = target.block_accumulator_info.num_leaves - 1;
    let target_block_accumulator = target.block_accumulator_info;

    let current_block_accumulator_info = current_block_info.block_accumulator_info.clone();

    let delay_milliseconds_on_error = 100;
    let sync_task = TaskGenerator::new(
        FindAncestorTask::new(
            current_block_number,
            target_block_number,
            10,
            fetcher.clone(),
        ),
        3,
        max_retry_times,
        delay_milliseconds_on_error,
        AncestorCollector::new(Arc::new(MerkleAccumulator::new_with_info(
            current_block_accumulator_info,
            storage.get_accumulator_store(AccumulatorStoreType::Block),
        ))),
        event_handle.clone(),
    )
    .generate();
    let (fut, _) = sync_task.with_handle();

    let (peer_sender, mut peer_receiver) = unbounded::<PeerEvent>();
    let peer_event_handle = PeerEventHandle::new(peer_sender);

    let event_handle_clone = event_handle.clone();

    let all_fut = async move {
        let ancestor = fut.await?;
        let mut ancestor_event_handle = ancestor_event_handle;
        if let Err(e) = ancestor_event_handle.handle(AncestorEvent { ancestor }) {
            error!(
                "Send AncestorEvent error: {:?}, ancestor: {:?}",
                e, ancestor
            );
        }
        let mut latest_ancestor = ancestor;
        let mut latest_block_chain;
        let start_time = get_unix_ts_as_millis();

        loop {
            while let Ok(Some(peer_event)) = peer_receiver.try_next() {
                if let PeerEvent::Open(peer_id, chain_info) = peer_event {
                    fetcher.new_peer(PeerInfo::new(peer_id, *chain_info));
                }
            }

            // sub target
            let target_number = latest_ancestor.number + 1000;
            let sub_target_task = FindSubTargetTask::new(fetcher.clone(), target_number);
            let (peers, sub_target) = sub_target_task
                .sub_target()
                .await
                .map_err(TaskError::BreakError)?;
            fetcher.filter(&peers);

            let real_target = match sub_target {
                None => target_block_accumulator.clone(),
                Some((_, target)) => target,
            };
            let inner = InnerSyncTask::new(
                latest_ancestor,
                real_target,
                storage.clone(),
                block_event_handle.clone(),
                fetcher.clone(),
                event_handle_clone.clone(),
                time_service.clone(),
                network.clone(),
            );
            let (block_chain, _) = inner
                .do_sync(
                    current_block_info.clone(),
                    5,
                    max_retry_times,
                    delay_milliseconds_on_error,
                    skip_pow_verify,
                )
                .await?;
            latest_block_chain = block_chain;
            let total_num = latest_block_chain.current_header().number() - ancestor.number;
            let total_time = get_unix_ts_as_millis() - start_time;
            info!(
                "sync strategy : {:?}, sync blocks: {:?}, time : {:?}, avg: {:?}",
                strategy,
                total_num,
                total_time,
                total_time / total_num as u128
            );
            if target_block_accumulator == latest_block_chain.current_block_accumulator_info() {
                break;
            }
            latest_ancestor = latest_block_chain.current_header().into();
        }
        Ok(latest_block_chain)
    };
    let task = TaskFuture::new(all_fut.boxed());
    let (fut, handle) = task.with_handle();
    Ok((fut, handle, event_handle, peer_event_handle))
}
