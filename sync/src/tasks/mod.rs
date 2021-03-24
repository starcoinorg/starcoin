// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::block_sync_task::SyncBlockData;
use crate::tasks::inner_sync_task::InnerSyncTask;
use crate::verified_rpc_client::{RpcVerifyError, VerifiedRpcClient};
use anyhow::{format_err, Error, Result};
use futures::channel::mpsc::UnboundedSender;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use logger::prelude::*;
use network_api::{PeerProvider, PeerSelector};
use network_rpc_core::{NetRpcError, RpcErrorCode};
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::MerkleAccumulator;
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, EventHandler, ServiceRef};
use starcoin_storage::Store;
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{Block, BlockIdAndNumber, BlockInfo, BlockNumber};
use starcoin_types::peer_info::PeerId;
use starcoin_vm_types::time::TimeService;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Instant;
use stream_task::{
    CustomErrorHandle, Generator, TaskError, TaskEventCounterHandle, TaskFuture, TaskGenerator,
    TaskHandle,
};

pub trait SyncFetcher: PeerOperator + BlockIdFetcher + BlockFetcher + BlockInfoFetcher {}

impl<T> SyncFetcher for Arc<T> where T: SyncFetcher {}

pub trait PeerOperator: Send + Sync {
    fn peer_selector(&self) -> PeerSelector;
}

pub trait BlockIdFetcher: Send + Sync {
    fn fetch_block_ids(
        &self,
        peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>>;

    fn fetch_block_id(
        &self,
        peer: Option<PeerId>,
        number: BlockNumber,
    ) -> BoxFuture<Result<Option<HashValue>>> {
        self.fetch_block_ids(peer, number, false, 1)
            .and_then(|mut ids| async move { Ok(ids.pop()) })
            .boxed()
    }
}

impl PeerOperator for VerifiedRpcClient {
    fn peer_selector(&self) -> PeerSelector {
        self.selector().clone()
    }
}

fn fetcher_err_map(err: Error) -> Error {
    match err.downcast::<RpcVerifyError>() {
        Ok(err) => TaskError::BreakError(err.into()).into(),
        Err(err) => err,
    }
}

impl BlockIdFetcher for VerifiedRpcClient {
    fn fetch_block_ids(
        &self,
        peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        self.get_block_ids(peer, start_number, reverse, max_size)
            .map_err(fetcher_err_map)
            .boxed()
    }
}

impl<T> PeerOperator for Arc<T>
where
    T: PeerOperator,
{
    fn peer_selector(&self) -> PeerSelector {
        PeerOperator::peer_selector(self.as_ref())
    }
}

impl<T> BlockIdFetcher for Arc<T>
where
    T: BlockIdFetcher,
{
    fn fetch_block_ids(
        &self,
        peer: Option<PeerId>,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        BlockIdFetcher::fetch_block_ids(self.as_ref(), peer, start_number, reverse, max_size)
    }
}

pub trait BlockFetcher: Send + Sync {
    fn fetch_blocks(
        &self,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<(Block, Option<PeerId>)>>>;
}

impl<T> BlockFetcher for Arc<T>
where
    T: BlockFetcher,
{
    fn fetch_blocks(
        &self,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<'_, Result<Vec<(Block, Option<PeerId>)>>> {
        BlockFetcher::fetch_blocks(self.as_ref(), block_ids)
    }
}

impl BlockFetcher for VerifiedRpcClient {
    fn fetch_blocks(
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
                results.map_err(fetcher_err_map)
            })
            .boxed()
    }
}

pub trait BlockInfoFetcher: Send + Sync {
    fn fetch_block_infos(
        &self,
        peer_id: Option<PeerId>,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<BlockInfo>>>>;
    fn fetch_block_info(
        &self,
        peer_id: Option<PeerId>,
        block_id: HashValue,
    ) -> BoxFuture<Result<Option<BlockInfo>>> {
        self.fetch_block_infos(peer_id, vec![block_id])
            .and_then(|mut block_infos| async move { Ok(block_infos.pop().flatten()) })
            .boxed()
    }
}

impl<T> BlockInfoFetcher for Arc<T>
where
    T: BlockInfoFetcher,
{
    fn fetch_block_infos(
        &self,
        peer_id: Option<PeerId>,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<BlockInfo>>>> {
        BlockInfoFetcher::fetch_block_infos(self.as_ref(), peer_id, block_ids)
    }
}

impl BlockInfoFetcher for VerifiedRpcClient {
    fn fetch_block_infos(
        &self,
        peer_id: Option<PeerId>,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<'_, Result<Vec<Option<BlockInfo>>>> {
        self.get_block_infos_from_peer(peer_id, block_ids)
            .map_err(fetcher_err_map)
            .boxed()
    }
}

impl SyncFetcher for VerifiedRpcClient {}

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

pub struct ExtSyncTaskErrorHandle<F>
where
    F: SyncFetcher + 'static,
{
    fetcher: Arc<F>,
}

impl<F> ExtSyncTaskErrorHandle<F>
where
    F: SyncFetcher + 'static,
{
    fn new(fetcher: Arc<F>) -> Self {
        Self { fetcher }
    }
}

impl<F> CustomErrorHandle for ExtSyncTaskErrorHandle<F>
where
    F: SyncFetcher + 'static,
{
    fn handle(&self, error: Error) {
        let peer_str = error.to_string();
        debug!("[sync]sync task peer_str: {:?}", peer_str);
        if let Ok(peer_id) = PeerId::from_str(&peer_str) {
            if let Ok(prc_error) = error.downcast::<NetRpcError>() {
                match &prc_error.error_code() {
                    RpcErrorCode::Forbidden
                    | RpcErrorCode::MethodNotFound
                    | RpcErrorCode::ServerUnavailable
                    | RpcErrorCode::Unknown
                    | RpcErrorCode::InternalError => {
                        let peers = self.fetcher.peer_selector().remove_peer(&peer_id);
                        debug!("[sync]sync task, peer len {}", peers);
                    }
                    _ => {
                        debug!("[sync]sync task err: {:?}", prc_error);
                    }
                }
            }
        }
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

use crate::tasks::sync_score_metrics::SYNC_SCORE_METRICS;
pub use accumulator_sync_task::{AccumulatorCollector, BlockAccumulatorSyncTask};
pub use block_sync_task::{BlockCollector, BlockSyncTask};
pub use find_ancestor_task::{AncestorCollector, FindAncestorTask};
use std::str::FromStr;

pub fn full_sync_task<H, A, F, N>(
    current_block_id: HashValue,
    target: SyncTarget,
    skip_pow_verify: bool,
    time_service: Arc<dyn TimeService>,
    storage: Arc<dyn Store>,
    block_event_handle: H,
    fetcher: Arc<F>,
    ancestor_event_handle: A,
    peer_provider: N,
    max_retry_times: u64,
) -> Result<(
    BoxFuture<'static, Result<BlockChain, TaskError>>,
    TaskHandle,
    Arc<TaskEventCounterHandle>,
)>
where
    H: BlockConnectedEventHandle + Sync + 'static,
    A: AncestorEventHandle + Sync + 'static,
    F: SyncFetcher + 'static,
    N: PeerProvider + Clone + 'static,
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

    let target_block_number = target.target_id.number();

    let current_block_accumulator_info = current_block_info.block_accumulator_info.clone();

    let delay_milliseconds_on_error = 100;
    //only keep the best peer for find ancestor.
    fetcher.peer_selector().retain(target.peers.as_slice());
    let ext_error_handle = Arc::new(ExtSyncTaskErrorHandle::new(fetcher.clone()));

    let sync_task = TaskGenerator::new(
        FindAncestorTask::new(
            current_block_number,
            target_block_number,
            10,
            fetcher.clone(),
        ),
        2,
        max_retry_times,
        delay_milliseconds_on_error,
        AncestorCollector::new(Arc::new(MerkleAccumulator::new_with_info(
            current_block_accumulator_info,
            storage.get_accumulator_store(AccumulatorStoreType::Block),
        ))),
        event_handle.clone(),
        ext_error_handle.clone(),
    )
    .generate();
    let (fut, _) = sync_task.with_handle();

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

        loop {
            // for get new peers from network.
            let all_peers = peer_provider
                .peer_set()
                .await
                .map_err(TaskError::BreakError)?;
            for peer in all_peers {
                fetcher.peer_selector().add_or_update_peer(peer);
            }
            let sub_target =
                inner_sync_task::sub_target(target.clone(), latest_ancestor, fetcher.clone())
                    .await
                    .map_err(TaskError::BreakError)?;

            fetcher.peer_selector().retain(sub_target.peers.as_slice());

            let inner = InnerSyncTask::new(
                latest_ancestor,
                sub_target,
                storage.clone(),
                block_event_handle.clone(),
                fetcher.clone(),
                event_handle_clone.clone(),
                time_service.clone(),
                peer_provider.clone(),
                ext_error_handle.clone(),
            );
            let start_now = Instant::now();
            let (block_chain, _) = inner
                .do_sync(
                    current_block_info.clone(),
                    2,
                    max_retry_times,
                    delay_milliseconds_on_error,
                    skip_pow_verify,
                )
                .await?;
            let total_time = Instant::now()
                .saturating_duration_since(start_now)
                .as_millis();
            latest_block_chain = block_chain;
            let total_num = latest_block_chain
                .current_header()
                .number()
                .saturating_sub(latest_ancestor.number);
            info!(
                "[sync] sync strategy : {:?}, sync blocks: {:?}, time : {:?}, avg: {:?}",
                fetcher.peer_selector().strategy(),
                total_num,
                total_time,
                total_time.checked_div(total_num as u128).unwrap_or(0)
            );

            SYNC_SCORE_METRICS.report_sub_sync_target_metrics(
                fetcher.peer_selector().len(),
                fetcher.peer_selector().strategy(),
                total_num as i64,
                total_time as i64,
            );
            if target.target_id.number() <= latest_block_chain.status().head.number() {
                break;
            }
            latest_ancestor = latest_block_chain.current_header().into();
        }
        Ok(latest_block_chain)
    };
    let task = TaskFuture::new(all_fut.boxed());
    let (fut, handle) = task.with_handle();
    Ok((fut, handle, event_handle))
}
