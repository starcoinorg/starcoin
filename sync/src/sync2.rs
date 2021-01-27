// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::BlockConnectorService;
use crate::peer_event_handle::PeerEventHandle;
use crate::tasks::{full_sync_task, AncestorEvent};
use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::{format_err, Result};
use chain::BlockChain;
use config::NodeConfig;
use futures::FutureExt;
use futures_timer::Delay;
use logger::prelude::*;
use network::NetworkServiceRef;
use network::PeerEvent;
use network_api::{PeerProvider, PeerSelector, PeerStrategy};
use starcoin_chain_api::ChainReader;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler,
};
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::{
    PeerScoreRequest, PeerScoreResponse, SyncCancelRequest, SyncProgressReport,
    SyncProgressRequest, SyncServiceHandler, SyncStartRequest, SyncStatusRequest, SyncTarget,
};
use starcoin_types::block::BlockIdAndNumber;
use starcoin_types::peer_info::PeerId;
use starcoin_types::startup_info::ChainStatus;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::{NewHeadBlock, SyncStatusChangeEvent, SystemStarted};
use std::sync::Arc;
use std::time::Duration;
use stream_task::{TaskError, TaskEventCounterHandle, TaskHandle};

//TODO combine task_handle and task_event_handle in stream_task
pub struct SyncTaskHandle {
    target: SyncTarget,
    task_begin: Option<BlockIdAndNumber>,
    task_handle: TaskHandle,
    task_event_handle: Arc<TaskEventCounterHandle>,
    peer_event_handle: PeerEventHandle,
    peer_selector: PeerSelector,
}

pub enum SyncStage {
    NotStart,
    Checking,
    Synchronizing(Box<SyncTaskHandle>),
    Canceling,
    Done,
}

pub struct SyncService2 {
    sync_status: SyncStatus,
    stage: SyncStage,
    config: Arc<NodeConfig>,
    storage: Arc<Storage>,
}

impl SyncService2 {
    pub fn new(config: Arc<NodeConfig>, storage: Arc<Storage>) -> Result<Self> {
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("can't get startup info"))?;
        let head_block_hash = startup_info.main;
        let head_block = storage
            .get_block(head_block_hash)?
            .ok_or_else(|| format_err!("can't get block by hash {}", head_block_hash))?;
        let head_block_info = storage
            .get_block_info(head_block_hash)?
            .ok_or_else(|| format_err!("can't get block info by hash {}", head_block_hash))?;

        Ok(Self {
            sync_status: SyncStatus::new(ChainStatus::new(head_block.header, head_block_info)),
            stage: SyncStage::NotStart,
            config,
            storage,
        })
    }

    pub fn check_and_start_sync(
        &mut self,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
        peer_strategy: Option<PeerStrategy>,
        ctx: &mut ServiceContext<Self>,
    ) -> Result<()> {
        match std::mem::replace(&mut self.stage, SyncStage::Checking) {
            SyncStage::NotStart | SyncStage::Done => {
                //continue
                info!(
                    "[sync] Start checking sync,skip_pow_verify:{}, special peers: {:?}",
                    skip_pow_verify, peers
                );
            }
            SyncStage::Checking => {
                info!("[sync] Sync stage is already in Checking");
                return Ok(());
            }
            SyncStage::Synchronizing(task_handle) => {
                info!("[sync] Sync stage is already in Synchronizing");
                if let Some(report) = task_handle.task_event_handle.get_report() {
                    info!("[sync] report: {}", report);
                }
                //restore to Synchronizing
                self.stage = SyncStage::Synchronizing(task_handle);
                return Ok(());
            }
            SyncStage::Canceling => {
                info!("[sync] Sync task is in canceling.");
                return Ok(());
            }
        }

        let network = ctx.get_shared::<NetworkServiceRef>()?;
        let storage = self.storage.clone();
        let self_ref = ctx.self_ref();
        let connector_service = ctx.service_ref::<BlockConnectorService>()?.clone();
        let config = self.config.clone();
        let fut = async move {
            let mut peer_selector = network.peer_selector().await?;
            loop {
                if peer_selector.is_empty()
                    || peer_selector.len() < (config.net().min_peers() as usize)
                {
                    info!(
                        "[sync]Wait enough peers {:?} : {:?}",
                        peer_selector.len(),
                        config.net().min_peers()
                    );
                    Delay::new(Duration::from_secs(1)).await;
                    peer_selector = network.peer_selector().await?;
                } else {
                    break;
                }
            }

            if !peers.is_empty() {
                peer_selector.retain(peers.as_ref())
            }
            if peer_selector.is_empty() {
                //info!("[sync] No peers to sync.");
                return Err(format_err!("[sync] No peers to sync."));
            }
            let rpc_client = VerifiedRpcClient::new(peer_selector, network.clone());
            let target = rpc_client.get_sync_target().await?;

            let startup_info = storage
                .get_startup_info()?
                .ok_or_else(|| format_err!("Startup info should exist."))?;
            let current_block_id = startup_info.main;
            let current_block_info =
                storage.get_block_info(current_block_id)?.ok_or_else(|| {
                    format_err!("Can not find block info by id: {}", current_block_id)
                })?;
            info!("[sync] Find target({}), total_difficulty:{}, current head({})'s total_difficulty({})", target.block_header.id(), target.block_info.total_difficulty, current_block_id, current_block_info.total_difficulty);
            if current_block_info.total_difficulty >= target.block_info.total_difficulty {
                info!("[sync] Current is already bast.");
                return Ok(None);
            }

            let peer_select_strategy = peer_strategy.unwrap_or_default();
            let peer_selector =
                PeerSelector::new(target.peers.clone(), peer_select_strategy.clone());
            let rpc_client = Arc::new(VerifiedRpcClient::new(
                peer_selector.clone(),
                network.clone(),
            ));

            let (fut, task_handle, task_event_handle, peer_event_handle) = full_sync_task(
                current_block_id,
                target.block_info.clone(),
                skip_pow_verify,
                config.net().time_service(),
                storage.clone(),
                connector_service.clone(),
                rpc_client,
                self_ref.clone(),
                network.clone(),
                config.sync.max_retry_times(),
                peer_select_strategy,
            )?;

            self_ref.notify(SyncBeginEvent {
                target,
                task_handle,
                task_event_handle,
                peer_event_handle,
                peer_selector,
            })?;
            Ok(Some(fut.await?))
            //Ok(())
        };
        let self_ref = ctx.self_ref();
        ctx.spawn(fut.then(
            |result: Result<Option<BlockChain>, anyhow::Error>| async move {
                let cancel = match result {
                    Ok(Some(chain)) => {
                        info!("[sync] Sync to latest block: {:?}", chain.current_header());
                        false
                    }
                    Ok(None) => {
                        debug!("[sync] Check sync task return none, do not need sync.");
                        false
                    }
                    Err(err) => {
                        if let Some(task_err) = err.downcast_ref::<TaskError>() {
                            info!("[sync] Sync task is cancel");
                            task_err.is_canceled()
                        } else {
                            error!("[sync] Sync task error: {:?}", err);
                            false
                        }
                    }
                };
                if let Err(e) = self_ref.notify(SyncDoneEvent { cancel }) {
                    error!("[sync] Broadcast SyncDone event error: {:?}", e);
                }
            },
        ));
        Ok(())
    }

    fn task_handle(&self) -> Option<&SyncTaskHandle> {
        match &self.stage {
            SyncStage::Synchronizing(handle) => Some(handle),
            _ => None,
        }
    }

    fn cancel_task(&mut self) {
        match std::mem::replace(&mut self.stage, SyncStage::Canceling) {
            SyncStage::Synchronizing(handle) => handle.task_handle.cancel(),
            stage => {
                //restore state machine state.
                self.stage = stage;
            }
        }
    }
}

impl ServiceFactory<Self> for SyncService2 {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<SyncService2> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;

        Self::new(config, storage)
    }
}

impl ActorService for SyncService2 {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SystemStarted>();
        ctx.subscribe::<PeerEvent>();
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SystemStarted>();
        ctx.unsubscribe::<PeerEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, AncestorEvent> for SyncService2 {
    fn handle_event(&mut self, msg: AncestorEvent, _ctx: &mut ServiceContext<SyncService2>) {
        match &mut self.stage {
            SyncStage::Synchronizing(handle) => {
                handle.task_begin = Some(msg.ancestor);
            }
            _ => {
                warn!("[sync] Invalid state, Receive AncestorEvent, but sync state is not Synchronizing.");
            }
        }
    }
}

impl EventHandler<Self, PeerEvent> for SyncService2 {
    fn handle_event(&mut self, msg: PeerEvent, ctx: &mut ServiceContext<Self>) {
        if self.sync_status.is_prepare() {
            return;
        }

        if let SyncStage::Synchronizing(task_handle) = &mut self.stage {
            if let Err(e) = task_handle.peer_event_handle.push(msg.clone()) {
                error!("[sync] Push PeerEvent error: {:?}", e);
            }
        }

        match msg {
            PeerEvent::Open(open_peer_id, _) => {
                debug!("[sync] connect new peer:{:?}", open_peer_id);
                ctx.notify(CheckSyncEvent::default());
            }
            PeerEvent::Close(close_peer_id) => {
                debug!("[sync] disconnect peer: {:?}", close_peer_id);
                if let Some(task_handle) = self.task_handle() {
                    if task_handle
                        .target
                        .peers
                        .iter()
                        .any(|peer| peer.peer_id() == close_peer_id)
                    {
                        warn!(
                            "[sync] Current sync task may be failed because peer {} closed",
                            close_peer_id
                        );
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncBeginEvent {
    target: SyncTarget,
    task_handle: TaskHandle,
    task_event_handle: Arc<TaskEventCounterHandle>,
    peer_event_handle: PeerEventHandle,
    peer_selector: PeerSelector,
}

impl EventHandler<Self, SyncBeginEvent> for SyncService2 {
    fn handle_event(&mut self, msg: SyncBeginEvent, ctx: &mut ServiceContext<Self>) {
        let (target, task_handle, task_event_handle, peer_event_handle, peer_selector) = (
            msg.target,
            msg.task_handle,
            msg.task_event_handle,
            msg.peer_event_handle,
            msg.peer_selector,
        );
        let sync_task_handle = SyncTaskHandle {
            target: target.clone(),
            task_begin: None,
            task_handle: task_handle.clone(),
            task_event_handle,
            peer_event_handle,
            peer_selector,
        };
        match std::mem::replace(
            &mut self.stage,
            SyncStage::Synchronizing(Box::new(sync_task_handle)),
        ) {
            SyncStage::NotStart | SyncStage::Done => {
                warn!(
                    "[sync] Unexpect SyncBeginEvent, current stage is NotStart|Done, expect: Checking."
                );
                //TODO should cancel task and restore state.
                //self.stage = SyncStage::NotStart;
                //task_handle.cancel();
            }
            SyncStage::Checking => {
                let target_total_difficulty = target.block_info.total_difficulty;
                let current_total_difficulty = self.sync_status.chain_status().total_difficulty();
                if target_total_difficulty <= current_total_difficulty {
                    info!("[sync] target block({})'s total_difficulty({}) is <= current's total_difficulty({}), cancel sync task.", target.block_header.number(), target_total_difficulty, current_total_difficulty);
                    self.stage = SyncStage::Done;
                    task_handle.cancel();
                } else {
                    let target_id_number = BlockIdAndNumber::new(
                        target.block_header.id(),
                        target.block_header.number(),
                    );
                    self.sync_status
                        .sync_begin(target_id_number, target.block_info.total_difficulty);
                    ctx.broadcast(SyncStatusChangeEvent(self.sync_status.clone()));
                }
            }
            SyncStage::Synchronizing(previous_handle) => {
                //this should not happen.
                warn!(
                    "[sync] Unexpect SyncBeginEvent, current stage is Synchronizing(target: {:?})",
                    previous_handle.target
                );
                //restore to previous and cancel new handle.
                self.stage = SyncStage::Synchronizing(previous_handle);
                task_handle.cancel();
            }
            SyncStage::Canceling => {
                self.stage = SyncStage::Canceling;
                task_handle.cancel();
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CheckSyncEvent {
    /// check sync with special peers
    peers: Vec<PeerId>,

    skip_pow_verify: bool,

    strategy: Option<PeerStrategy>,
}

impl CheckSyncEvent {
    pub fn new(peers: Vec<PeerId>, skip_pow_verify: bool, strategy: Option<PeerStrategy>) -> Self {
        Self {
            peers,
            skip_pow_verify,
            strategy,
        }
    }
}

impl EventHandler<Self, CheckSyncEvent> for SyncService2 {
    fn handle_event(&mut self, msg: CheckSyncEvent, ctx: &mut ServiceContext<Self>) {
        if let Err(e) = self.check_and_start_sync(msg.peers, msg.skip_pow_verify, msg.strategy, ctx)
        {
            error!("[sync] Check sync error: {:?}", e);
        };
    }
}

impl EventHandler<Self, SystemStarted> for SyncService2 {
    fn handle_event(&mut self, _msg: SystemStarted, ctx: &mut ServiceContext<Self>) {
        // change from prepare to Synchronized
        self.sync_status.sync_done();
        ctx.notify(CheckSyncEvent::default());
        ctx.broadcast(SyncStatusChangeEvent(self.sync_status.clone()));
    }
}

#[derive(Clone, Debug)]
pub struct SyncDoneEvent {
    cancel: bool,
}

impl EventHandler<Self, SyncDoneEvent> for SyncService2 {
    fn handle_event(&mut self, _msg: SyncDoneEvent, ctx: &mut ServiceContext<Self>) {
        match std::mem::replace(&mut self.stage, SyncStage::Done) {
            SyncStage::NotStart | SyncStage::Done => {
                warn!(
                    "[sync] Unexpect sync stage, current is NotStart|Done, but got SyncDoneEvent"
                );
            }
            SyncStage::Checking => debug!("[sync] Sync task is Done in checking stage."),
            SyncStage::Synchronizing(task_handle) => {
                if !task_handle.task_handle.is_done() {
                    warn!(
                        "[sync] Current SyncStatus is invalid, receive sync done event ,but sync task not done.",
                    )
                }
                self.sync_status.sync_done();
                ctx.broadcast(SyncStatusChangeEvent(self.sync_status.clone()));
                // check sync again
                //TODO do not broadcast SyncDone, if node still not synchronized after check sync.
                ctx.notify(CheckSyncEvent::default());
            }
            SyncStage::Canceling => {
                //continue
                self.sync_status.sync_done();
                ctx.broadcast(SyncStatusChangeEvent(self.sync_status.clone()));
            }
        }
    }
}

impl EventHandler<Self, NewHeadBlock> for SyncService2 {
    fn handle_event(&mut self, msg: NewHeadBlock, ctx: &mut ServiceContext<Self>) {
        let NewHeadBlock(block) = msg;
        if self.sync_status.update_chain_status(ChainStatus::new(
            block.header().clone(),
            block.block_info.clone(),
        )) {
            ctx.broadcast(SyncStatusChangeEvent(self.sync_status.clone()));
        }
    }
}

impl ServiceHandler<Self, SyncStatusRequest> for SyncService2 {
    fn handle(
        &mut self,
        _msg: SyncStatusRequest,
        _ctx: &mut ServiceContext<SyncService2>,
    ) -> SyncStatus {
        self.sync_status.clone()
    }
}

impl ServiceHandler<Self, PeerScoreRequest> for SyncService2 {
    fn handle(
        &mut self,
        _msg: PeerScoreRequest,
        _ctx: &mut ServiceContext<SyncService2>,
    ) -> PeerScoreResponse {
        let resp = match &mut self.stage {
            SyncStage::Synchronizing(handle) => Some(handle.peer_selector.scores()),
            _ => None,
        };
        resp.into()
    }
}

impl ServiceHandler<Self, SyncProgressRequest> for SyncService2 {
    fn handle(
        &mut self,
        _msg: SyncProgressRequest,
        _ctx: &mut ServiceContext<SyncService2>,
    ) -> Option<SyncProgressReport> {
        self.task_handle().and_then(|handle| {
            handle.task_event_handle.total_report().map(|mut report| {
                if let Some(begin) = handle.task_begin.as_ref() {
                    report.fix_percent(handle.target.block_header.number() - begin.number);
                }

                SyncProgressReport {
                    target_id: handle.target.block_header.id(),
                    begin_number: handle
                        .task_begin
                        .as_ref()
                        .map(|begin| -> u64 { begin.number }),
                    target_number: handle.target.block_header.number(),
                    target_difficulty: handle.target.block_info.total_difficulty,
                    target_peers: handle
                        .target
                        .peers
                        .iter()
                        .map(|peer| peer.peer_id())
                        .collect(),
                    current: report,
                }
            })
        })
    }
}

impl ServiceHandler<Self, SyncCancelRequest> for SyncService2 {
    fn handle(&mut self, _msg: SyncCancelRequest, _ctx: &mut ServiceContext<SyncService2>) {
        self.cancel_task();
    }
}

impl ServiceHandler<Self, SyncStartRequest> for SyncService2 {
    fn handle(
        &mut self,
        msg: SyncStartRequest,
        ctx: &mut ServiceContext<SyncService2>,
    ) -> Result<()> {
        if msg.force {
            info!("[sync] Try to cancel previous sync task, because receive force sync request.");
            self.cancel_task();
        }
        ctx.notify(CheckSyncEvent::new(
            msg.peers,
            msg.skip_pow_verify,
            msg.strategy,
        ));
        Ok(())
    }
}

impl SyncServiceHandler for SyncService2 {}
