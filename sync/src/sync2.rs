// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::BlockConnectorService;
use crate::tasks::full_sync_task;
use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::{format_err, Result};
use config::NodeConfig;
use futures::FutureExt;
use logger::prelude::*;
use network::NetworkServiceRef;
use network::PeerEvent;
use network_api::{PeerProvider, PeerSelector};
use starcoin_chain_api::ChainReader;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler,
};
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::{
    SyncCancelRequest, SyncProgressReport, SyncProgressRequest, SyncServiceHandler,
    SyncStartRequest, SyncStatusRequest, SyncTarget,
};
use starcoin_types::block::BlockIdAndNumber;
use starcoin_types::peer_info::PeerId;
use starcoin_types::startup_info::ChainStatus;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::{NewHeadBlock, SyncStatusChangeEvent, SystemStarted};
use std::sync::Arc;
use stream_task::{TaskEventCounterHandle, TaskHandle};

//TODO combine task_handle and task_event_handle in stream_task
pub struct SyncTaskHandle {
    target: SyncTarget,
    task_handle: TaskHandle,
    task_event_handle: Arc<TaskEventCounterHandle>,
}

pub struct SyncService2 {
    sync_status: SyncStatus,
    config: Arc<NodeConfig>,
    storage: Arc<Storage>,
    task_handle: Option<SyncTaskHandle>,
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
            sync_status: SyncStatus::new(ChainStatus::new(
                head_block.header,
                head_block_info.total_difficulty,
            )),
            config,
            storage,
            task_handle: None,
        })
    }

    pub fn check_sync(
        &self,
        force: bool,
        peers: Vec<PeerId>,
        ctx: &mut ServiceContext<Self>,
    ) -> Result<()> {
        if let Some(task_handle) = self.task_handle.as_ref() {
            let task_running = !task_handle.task_handle.is_done();
            if task_running && force {
                info!("[sync] Cancel previous sync task.");
                task_handle.task_handle.cancel();
            } else if task_running && !force {
                debug!("[sync] Sync task is running");
                if let Some(report) = task_handle.task_event_handle.get_report() {
                    info!("[sync]{}", report);
                }
                return Ok(());
            }
        }
        let network = ctx.get_shared::<NetworkServiceRef>()?;
        let self_ref = ctx.self_ref();
        let fut = async move {
            let peer_selector = network.peer_selector().await?;
            if peer_selector.is_empty() {
                info!("[sync] No peers to sync.");
                return Ok(());
            }
            let peer_selector = if !peers.is_empty() {
                peer_selector
                    .selector()
                    .filter(move |peer_info| peers.contains(&peer_info.peer_id()))
                    .into_selector()
            } else {
                peer_selector
            };
            if peer_selector.is_empty() {
                info!("[sync] No peers to sync.");
                return Ok(());
            }
            let rpc_client = VerifiedRpcClient::new(peer_selector, network);
            let target = rpc_client.get_sync_target().await?;
            self_ref.notify(StartSyncEvent { target })?;
            Ok(())
        };
        ctx.spawn(fut.then(|result: Result<(), anyhow::Error>| async move {
            if let Err(e) = result {
                error!("[sync] Find best target task error: {}", e);
            }
        }));
        Ok(())
    }

    pub fn start_sync_task(
        &mut self,
        target: SyncTarget,
        ctx: &mut ServiceContext<Self>,
    ) -> Result<()> {
        if let Some(task_handle) = self.task_handle.as_ref() {
            let task_running = !task_handle.task_handle.is_done();
            if task_running {
                //TODO replace old task with new task at some condition.
                info!(
                    "[sync] A sync task is running，current target: {:?}, ignore new sync target: {:?}",
                    task_handle.target, target
                );
                return Ok(());
            }
        }
        let startup_info = self
            .storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist."))?;
        let current_block_id = startup_info.main;

        let network = ctx.get_shared::<NetworkServiceRef>()?;
        let peer_selector = PeerSelector::new(target.peers.clone());
        let rpc_client = VerifiedRpcClient::new(peer_selector, network.clone());
        let connector_service = ctx.service_ref::<BlockConnectorService>()?;
        let (fut, task_handle, task_event_handle) = full_sync_task(
            current_block_id,
            target.block_info.clone(),
            self.config.net().time_service(),
            self.storage.clone(),
            connector_service.clone(),
            rpc_client,
            network,
        )?;
        let target_id_number =
            BlockIdAndNumber::new(target.block_header.id(), target.block_header.number);
        self.sync_status
            .sync_begin(target_id_number, target.block_info.total_difficulty);

        self.task_handle = Some(SyncTaskHandle {
            target,
            task_handle,
            task_event_handle,
        });

        ctx.broadcast(SyncStatusChangeEvent(self.sync_status.clone()));

        let self_ref = ctx.self_ref();
        ctx.spawn(fut.then(|result| async move {
            match result {
                Ok(chain) => info!("[sync] Sync to latest block: {:?}", chain.current_header()),
                Err(err) => {
                    error!("[sync] Sync task error: {:?}", err);
                }
            }
            if let Err(e) = self_ref.notify(SyncDone) {
                error!("[sync] Broadcast SyncDone event error: {:?}", e);
            }
        }));
        Ok(())
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

impl EventHandler<Self, PeerEvent> for SyncService2 {
    fn handle_event(&mut self, msg: PeerEvent, ctx: &mut ServiceContext<Self>) {
        if self.sync_status.is_prepare() {
            return;
        }
        match msg {
            PeerEvent::Open(open_peer_id, _) => {
                debug!("[sync] connect new peer:{:?}", open_peer_id);
                ctx.notify(CheckSyncEvent::default());
            }
            PeerEvent::Close(close_peer_id) => {
                debug!("[sync] disconnect peer: {:?}", close_peer_id);
                if let Some(task_handle) = self.task_handle.as_mut() {
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
pub struct StartSyncEvent {
    target: SyncTarget,
}

impl EventHandler<Self, StartSyncEvent> for SyncService2 {
    fn handle_event(&mut self, msg: StartSyncEvent, ctx: &mut ServiceContext<Self>) {
        let target_block_header = msg.target.block_header.clone();
        let target_total_difficulty = msg.target.block_info.total_difficulty;
        let current_total_difficulty = self.sync_status.chain_status().total_difficulty();
        if target_total_difficulty <= current_total_difficulty {
            debug!("[sync] target block({})'s total_difficulty({}) is <= current's total_difficulty({}), ignore StartSyncEvent.", target_block_header.number, target_total_difficulty, current_total_difficulty);
            return;
        }
        if let Err(e) = self.start_sync_task(msg.target, ctx) {
            error!(
                "[sync] Start sync task error: {:?}, target: {:?}",
                e, target_block_header
            );
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CheckSyncEvent {
    force: bool,
    /// check sync with special peers
    peers: Vec<PeerId>,
}

impl CheckSyncEvent {
    pub fn new(force: bool, peers: Vec<PeerId>) -> Self {
        Self { force, peers }
    }
}

impl EventHandler<Self, CheckSyncEvent> for SyncService2 {
    fn handle_event(&mut self, msg: CheckSyncEvent, ctx: &mut ServiceContext<Self>) {
        if let Err(e) = self.check_sync(msg.force, msg.peers, ctx) {
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
pub struct SyncDone;

impl EventHandler<Self, SyncDone> for SyncService2 {
    fn handle_event(&mut self, _msg: SyncDone, ctx: &mut ServiceContext<Self>) {
        if !self.sync_status.is_syncing() {
            warn!(
                "[sync] Current SyncStatus is invalid, expect Synchronizing, but got: {:?}",
                self.sync_status.sync_status()
            )
        }
        self.sync_status.sync_done();
        ctx.broadcast(SyncStatusChangeEvent(self.sync_status.clone()));
        // check sync again
        //TODO do not broadcast SyncDone, if node still not synchronized after check sync.
        ctx.notify(CheckSyncEvent::new(false, vec![]));
    }
}

impl EventHandler<Self, NewHeadBlock> for SyncService2 {
    fn handle_event(&mut self, msg: NewHeadBlock, ctx: &mut ServiceContext<Self>) {
        let NewHeadBlock(block) = msg;
        if self.sync_status.update_chain_status(ChainStatus::new(
            block.header().clone(),
            block.total_difficulty(),
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

impl ServiceHandler<Self, SyncProgressRequest> for SyncService2 {
    fn handle(
        &mut self,
        _msg: SyncProgressRequest,
        _ctx: &mut ServiceContext<SyncService2>,
    ) -> Option<SyncProgressReport> {
        self.task_handle.as_ref().and_then(|handle| {
            handle
                .task_event_handle
                .get_report()
                .map(|report| SyncProgressReport {
                    target_id: handle.target.block_header.id(),
                    target_number: handle.target.block_header.number,
                    target_difficulty: handle.target.block_info.total_difficulty,
                    target_peers: handle
                        .target
                        .peers
                        .iter()
                        .map(|peer| peer.peer_id())
                        .collect(),
                    current: report,
                })
        })
    }
}

impl ServiceHandler<Self, SyncCancelRequest> for SyncService2 {
    fn handle(&mut self, _msg: SyncCancelRequest, _ctx: &mut ServiceContext<SyncService2>) {
        if let Some(handle) = self.task_handle.as_ref() {
            handle.task_handle.cancel()
        }
    }
}

impl ServiceHandler<Self, SyncStartRequest> for SyncService2 {
    fn handle(
        &mut self,
        msg: SyncStartRequest,
        ctx: &mut ServiceContext<SyncService2>,
    ) -> Result<()> {
        ctx.notify(CheckSyncEvent::new(msg.force, msg.peers));
        Ok(())
    }
}

impl SyncServiceHandler for SyncService2 {}
