// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::BlockConnectorService;
use crate::sync_metrics::SyncMetrics;
use crate::tasks::{full_sync_task, AncestorEvent, SyncFetcher};
use crate::verified_rpc_client::{RpcVerifyError, VerifiedRpcClient};
use anyhow::{format_err, Result};
use config::NodeConfig;
use executor::VMMetrics;
use futures::FutureExt;
use futures_timer::Delay;
use logger::prelude::*;
use network::NetworkServiceRef;
use network::PeerEvent;
use network_api::peer_score::PeerScoreMetrics;
use network_api::{PeerProvider, PeerSelector, PeerStrategy, ReputationChange};
use starcoin_chain::BlockChain;
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

const REPUTATION_THRESHOLD: i32 = -1000;

//TODO combine task_handle and task_event_handle in stream_task
pub struct SyncTaskHandle {
    target: SyncTarget,
    task_begin: Option<BlockIdAndNumber>,
    task_handle: TaskHandle,
    task_event_handle: Arc<TaskEventCounterHandle>,
    peer_selector: PeerSelector,
}

pub enum SyncStage {
    NotStart,
    Checking,
    Synchronizing(Box<SyncTaskHandle>),
    Canceling,
    Done,
}

pub struct SyncService {
    sync_status: SyncStatus,
    stage: SyncStage,
    config: Arc<NodeConfig>,
    storage: Arc<Storage>,
    metrics: Option<SyncMetrics>,
    peer_score_metrics: Option<PeerScoreMetrics>,
    vm_metrics: Option<VMMetrics>,
}

impl SyncService {
    pub fn new(
        config: Arc<NodeConfig>,
        storage: Arc<Storage>,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
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
        //TODO bail PrometheusError after use custom metrics registry.
        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| SyncMetrics::register(registry).ok());
        let peer_score_metrics = config
            .metrics
            .registry()
            .and_then(|registry| PeerScoreMetrics::register(registry).ok());
        Ok(Self {
            sync_status: SyncStatus::new(ChainStatus::new(head_block.header, head_block_info)),
            stage: SyncStage::NotStart,
            config,
            storage,
            metrics,
            peer_score_metrics,
            vm_metrics,
        })
    }

    pub fn check_and_start_sync(
        &mut self,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
        peer_strategy: Option<PeerStrategy>,
        ctx: &mut ServiceContext<Self>,
    ) -> Result<()> {
        let sync_task_total = self
            .metrics
            .as_ref()
            .map(|metrics| metrics.sync_task_total.clone());

        if let Some(sync_task_total) = sync_task_total.as_ref() {
            sync_task_total.with_label_values(&["check"]).inc();
        }
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
        let peer_score_metrics = self.peer_score_metrics.clone();
        let sync_metrics = self.metrics.clone();
        let vm_metrics = self.vm_metrics.clone();
        let fut = async move {
            let peer_select_strategy =
                peer_strategy.unwrap_or_else(|| config.sync.peer_select_strategy());

            let mut peer_set = network.peer_set().await?;

            loop {
                if peer_set.is_empty() || peer_set.len() < (config.net().min_peers() as usize) {
                    let level = if config.net().is_dev() || config.net().is_test() {
                        Level::Debug
                    } else {
                        Level::Info
                    };
                    log!(
                        level,
                        "[sync]Waiting enough peers to sync, current: {:?} peers, min peers: {:?}",
                        peer_set.len(),
                        config.net().min_peers()
                    );

                    Delay::new(Duration::from_secs(1)).await;
                    peer_set = network.peer_set().await?;
                } else {
                    break;
                }
            }

            let peer_reputations = network
                .reputations(REPUTATION_THRESHOLD)
                .await?
                .await?
                .into_iter()
                .map(|(peer, reputation)| {
                    (
                        peer,
                        (REPUTATION_THRESHOLD.abs().saturating_add(reputation)) as u64,
                    )
                })
                .collect();

            let peer_selector = PeerSelector::new_with_reputation(
                peer_reputations,
                peer_set,
                peer_select_strategy,
                peer_score_metrics,
            );

            peer_selector.retain_rpc_peers();
            if !peers.is_empty() {
                peer_selector.retain(peers.as_ref())
            }
            if peer_selector.is_empty() {
                return Err(format_err!("[sync] No peers to sync."));
            }

            let startup_info = storage
                .get_startup_info()?
                .ok_or_else(|| format_err!("Startup info should exist."))?;
            let current_block_id = startup_info.main;
            let current_block_info =
                storage.get_block_info(current_block_id)?.ok_or_else(|| {
                    format_err!("Can not find block info by id: {}", current_block_id)
                })?;

            let rpc_client = Arc::new(VerifiedRpcClient::new(
                peer_selector.clone(),
                network.clone(),
            ));
            if let Some(target) =
                rpc_client.get_best_target(current_block_info.get_total_difficulty())?
            {
                info!("[sync] Find target({}), total_difficulty:{}, current head({})'s total_difficulty({})", target.target_id.id(), target.block_info.total_difficulty, current_block_id, current_block_info.total_difficulty);

                let (fut, task_handle, task_event_handle) = full_sync_task(
                    current_block_id,
                    target.clone(),
                    skip_pow_verify,
                    config.net().time_service(),
                    storage.clone(),
                    connector_service.clone(),
                    rpc_client.clone(),
                    self_ref.clone(),
                    network.clone(),
                    config.sync.max_retry_times(),
                    sync_metrics.clone(),
                    vm_metrics.clone(),
                )?;

                self_ref.notify(SyncBeginEvent {
                    target,
                    task_handle,
                    task_event_handle,
                    peer_selector,
                })?;
                if let Some(sync_task_total) = sync_task_total.as_ref() {
                    sync_task_total.with_label_values(&["start"]).inc();
                }
                Ok(Some(fut.await?))
            } else {
                debug!("[sync]No best peer to request, current is beast.");
                Ok(None)
            }
        };
        let network = ctx.get_shared::<NetworkServiceRef>()?;
        let self_ref = ctx.self_ref();

        let sync_task_total = self
            .metrics
            .as_ref()
            .map(|metrics| metrics.sync_task_total.clone());
        let sync_task_break_total = self
            .metrics
            .as_ref()
            .map(|metrics| metrics.sync_task_break_total.clone());

        ctx.spawn(fut.then(
            |result: Result<Option<BlockChain>, anyhow::Error>| async move {
                let cancel = match result {
                    Ok(Some(chain)) => {
                        info!("[sync] Sync to latest block: {:?}", chain.current_header());
                        if let Some(sync_task_total) = sync_task_total.as_ref() {
                            sync_task_total.with_label_values(&["done"]).inc();
                        }
                        false
                    }
                    Ok(None) => {
                        debug!("[sync] Check sync task return none, do not need sync.");
                        false
                    }
                    Err(err) => {
                        if let Some(task_err) = err.downcast_ref::<TaskError>() {
                            match task_err {
                                TaskError::Canceled => {
                                    info!("[sync] Sync task is cancel");
                                    if let Some(sync_task_total) = sync_task_total.as_ref() {
                                        sync_task_total.with_label_values(&["cancel"]).inc();
                                    }
                                    true
                                }
                                TaskError::BreakError(err) => {
                                    let reason = if let Some(rpc_verify_err) =
                                        err.downcast_ref::<RpcVerifyError>()
                                    {
                                        for peer_id in rpc_verify_err.peers.as_slice() {
                                            network.report_peer(
                                                peer_id.clone(),
                                                ReputationChange::new_fatal("invalid_response"),
                                            )
                                        }
                                        "verify_err"
                                    }else if let Some(bcs_err) = err.downcast_ref::<bcs_ext::Error>(){
                                        warn!("[sync] bcs codec error, maybe network rpc protocol is not compat with other peers: {:?}", bcs_err);
                                        "bcs_err"
                                    } else {
                                        "other_err"
                                    };
                                    if let Some(sync_task_break_total) = sync_task_break_total.as_ref() {
                                        sync_task_break_total.with_label_values(&[reason]).inc();
                                    }
                                    warn!(
                                        "[sync] Sync task is interrupted by {:?}, cause:{:?} ",
                                        err,
                                        err.root_cause(),
                                    );
                                    if let Some(sync_task_total) = sync_task_total.as_ref() {
                                        sync_task_total.with_label_values(&["break"]).inc();
                                    }
                                    false
                                }
                                task_err => {
                                    error!("[sync] Sync task error: {:?}", task_err);
                                    if let Some(sync_task_total) = sync_task_total.as_ref() {
                                        sync_task_total.with_label_values(&["error"]).inc();
                                    }
                                    false
                                }
                            }
                        } else {
                            error!("[sync] Sync task error: {:?}", err);
                            if let Some(sync_task_total) = sync_task_total.as_ref() {
                                sync_task_total.with_label_values(&["error"]).inc();
                            }
                            false
                        }
                    }
                };
                if let Err(e) = self_ref.notify(SyncDoneEvent{ cancel }) {
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

impl ServiceFactory<Self> for SyncService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<SyncService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;
        Self::new(config, storage, vm_metrics)
    }
}

impl ActorService for SyncService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SystemStarted>();
        ctx.subscribe::<PeerEvent>();
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        self.cancel_task();
        ctx.unsubscribe::<SystemStarted>();
        ctx.unsubscribe::<PeerEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, AncestorEvent> for SyncService {
    fn handle_event(&mut self, msg: AncestorEvent, _ctx: &mut ServiceContext<SyncService>) {
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

impl EventHandler<Self, PeerEvent> for SyncService {
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
                if let Some(task_handle) = self.task_handle() {
                    if task_handle
                        .target
                        .peers
                        .iter()
                        .any(|peer_id| peer_id == &close_peer_id)
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
    peer_selector: PeerSelector,
}

impl EventHandler<Self, SyncBeginEvent> for SyncService {
    fn handle_event(&mut self, msg: SyncBeginEvent, ctx: &mut ServiceContext<Self>) {
        let (target, task_handle, task_event_handle, peer_selector) = (
            msg.target,
            msg.task_handle,
            msg.task_event_handle,
            msg.peer_selector,
        );
        let sync_task_handle = SyncTaskHandle {
            target: target.clone(),
            task_begin: None,
            task_handle: task_handle.clone(),
            task_event_handle,
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
                    info!("[sync] target block({})'s total_difficulty({}) is <= current's total_difficulty({}), cancel sync task.", target.target_id.number(), target_total_difficulty, current_total_difficulty);
                    task_handle.cancel();
                } else {
                    let target_id_number =
                        BlockIdAndNumber::new(target.target_id.id(), target.target_id.number());
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

impl EventHandler<Self, CheckSyncEvent> for SyncService {
    fn handle_event(&mut self, msg: CheckSyncEvent, ctx: &mut ServiceContext<Self>) {
        if let Err(e) = self.check_and_start_sync(msg.peers, msg.skip_pow_verify, msg.strategy, ctx)
        {
            error!("[sync] Check sync error: {:?}", e);
        };
    }
}

impl EventHandler<Self, SystemStarted> for SyncService {
    fn handle_event(&mut self, _msg: SystemStarted, ctx: &mut ServiceContext<Self>) {
        // change from prepare to Synchronized
        self.sync_status.sync_done();
        ctx.notify(CheckSyncEvent::default());
        ctx.broadcast(SyncStatusChangeEvent(self.sync_status.clone()));
    }
}

#[derive(Clone, Debug)]
pub struct SyncDoneEvent {
    #[allow(unused)]
    cancel: bool,
}

impl EventHandler<Self, SyncDoneEvent> for SyncService {
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

impl EventHandler<Self, NewHeadBlock> for SyncService {
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

impl ServiceHandler<Self, SyncStatusRequest> for SyncService {
    fn handle(
        &mut self,
        _msg: SyncStatusRequest,
        _ctx: &mut ServiceContext<SyncService>,
    ) -> SyncStatus {
        self.sync_status.clone()
    }
}

impl ServiceHandler<Self, PeerScoreRequest> for SyncService {
    fn handle(
        &mut self,
        _msg: PeerScoreRequest,
        _ctx: &mut ServiceContext<SyncService>,
    ) -> PeerScoreResponse {
        let resp = match &mut self.stage {
            SyncStage::Synchronizing(handle) => Some(handle.peer_selector.scores()),
            _ => None,
        };
        resp.into()
    }
}

impl ServiceHandler<Self, SyncProgressRequest> for SyncService {
    fn handle(
        &mut self,
        _msg: SyncProgressRequest,
        _ctx: &mut ServiceContext<SyncService>,
    ) -> Option<SyncProgressReport> {
        self.task_handle().and_then(|handle| {
            handle.task_event_handle.total_report().map(|mut report| {
                if let Some(begin) = handle.task_begin.as_ref() {
                    report.fix_percent(
                        handle
                            .target
                            .target_id
                            .number()
                            .saturating_sub(begin.number),
                    );
                }

                SyncProgressReport {
                    target_id: handle.target.target_id.id(),
                    begin_number: handle
                        .task_begin
                        .as_ref()
                        .map(|begin| -> u64 { begin.number }),
                    target_number: handle.target.target_id.number(),
                    target_difficulty: handle.target.block_info.total_difficulty,
                    target_peers: handle.target.peers.clone(),
                    current: report,
                }
            })
        })
    }
}

impl ServiceHandler<Self, SyncCancelRequest> for SyncService {
    fn handle(&mut self, _msg: SyncCancelRequest, _ctx: &mut ServiceContext<SyncService>) {
        self.cancel_task();
    }
}

impl ServiceHandler<Self, SyncStartRequest> for SyncService {
    fn handle(
        &mut self,
        msg: SyncStartRequest,
        ctx: &mut ServiceContext<SyncService>,
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

impl SyncServiceHandler for SyncService {}
