// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::{BlockConnectorService, ConnectBlockRequest};
use crate::tasks::{full_sync_task, SyncTarget};
use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::{format_err, Result};
use config::NodeConfig;
use futures::FutureExt;
use logger::prelude::*;
use network::NetworkAsyncService;
use network::PeerEvent;
use network_api::{PeerProvider, PeerSelector};
use starcoin_chain_api::{ChainReader, ConnectBlockError};
use starcoin_service_registry::bus::Bus;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::PeerNewBlock;
use starcoin_types::system_events::{SyncBegin, SyncDone};
use std::sync::Arc;
use stream_task::{TaskEventCounterHandle, TaskHandle};

//TODO combine task_handle and task_event_handle in stream_task
pub struct SyncTaskHandle {
    target: SyncTarget,
    task_handle: TaskHandle,
    task_event_handle: Arc<TaskEventCounterHandle>,
}

pub struct SyncService2 {
    config: Arc<NodeConfig>,
    network: NetworkAsyncService,
    storage: Arc<Storage>,
    connector_service: ServiceRef<BlockConnectorService>,
    task_handle: Option<SyncTaskHandle>,
}

impl SyncService2 {
    pub fn new(
        config: Arc<NodeConfig>,
        network: NetworkAsyncService,
        storage: Arc<Storage>,
        connector_service: ServiceRef<BlockConnectorService>,
    ) -> Self {
        Self {
            config,
            network,
            storage,
            connector_service,
            task_handle: None,
        }
    }

    pub fn check_sync(&self, force: bool, ctx: &mut ServiceContext<SyncService2>) -> Result<()> {
        if let Some(task_handle) = self.task_handle.as_ref() {
            let task_running = !task_handle.task_handle.is_done();
            if task_running && force {
                info!("Cancel previous sync task.");
                task_handle.task_handle.cancel();
            } else if task_running && !force {
                debug!("Sync task is running");
                if let Some(report) = task_handle.task_event_handle.get_report() {
                    info!("[sync]{}", report);
                }
                return Ok(());
            }
        }
        let network = self.network.clone();
        let self_ref = ctx.self_ref();
        let fut = async move {
            let peer_selector = network.peer_selector().await?;
            if peer_selector.is_empty() {
                info!("No peers to sync.");
                return Ok(());
            }
            let rpc_client = VerifiedRpcClient::new(peer_selector, network);
            //TODO optimize target selector
            let target = rpc_client.get_sync_target().await?;
            self_ref.notify(StartSyncEvent { target })?;
            Ok(())
        };
        ctx.spawn(fut.then(|result: Result<(), anyhow::Error>| async move {
            if let Err(e) = result {
                error!("Find best target task error: {}", e);
            }
        }));
        Ok(())
    }

    pub fn start_sync_task(
        &mut self,
        target: SyncTarget,
        ctx: &mut ServiceContext<SyncService2>,
    ) -> Result<()> {
        let startup_info = self
            .storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist."))?;
        let current_block_id = startup_info.master;
        let network = self.network.clone();
        let peer_selector = PeerSelector::new(target.peers.clone());
        let rpc_client = VerifiedRpcClient::new(peer_selector, network);
        let (fut, task_handle, task_event_handle) = full_sync_task(
            current_block_id,
            target.block_info.clone(),
            self.config.net().time_service(),
            self.storage.clone(),
            self.connector_service.clone(),
            rpc_client,
        )?;
        self.task_handle = Some(SyncTaskHandle {
            target,
            task_handle,
            task_event_handle,
        });
        ctx.broadcast(SyncBegin);
        let bus = ctx.bus_ref().clone();
        ctx.spawn(fut.then(|result| async move {
            //TODO process sync result;
            match result {
                Ok(chain) => info!("Sync to latest block: {:?}", chain.current_header()),
                Err(err) => {
                    error!("Sync task error: {:?}", err);
                }
            }
            if let Err(e) = bus.broadcast(SyncDone) {
                error!("Broadcast SyncDone event error: {:?}", e);
            }
        }));
        Ok(())
    }
}

impl ServiceFactory<Self> for SyncService2 {
    fn create(ctx: &mut ServiceContext<SyncService2>) -> Result<SyncService2> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let network = ctx.get_shared::<NetworkAsyncService>()?;
        let connect_service = ctx.service_ref::<BlockConnectorService>()?;
        //let peer_id = node_config.network.self_peer_id()?;
        Ok(Self::new(config, network, storage, connect_service.clone()))
    }
}

impl ActorService for SyncService2 {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<PeerEvent>();
        ctx.subscribe::<PeerNewBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<PeerEvent>();
        ctx.unsubscribe::<PeerNewBlock>();
        Ok(())
    }
}

impl EventHandler<Self, PeerEvent> for SyncService2 {
    fn handle_event(&mut self, msg: PeerEvent, ctx: &mut ServiceContext<SyncService2>) {
        match msg {
            PeerEvent::Open(open_peer_id, _) => {
                debug!("connect new peer:{:?}", open_peer_id);
                if let Err(e) = self.check_sync(false, ctx) {
                    error!("Check sync error: {:?}", e);
                };
            }
            PeerEvent::Close(close_peer_id) => {
                debug!("disconnect peer: {:?}", close_peer_id);
                if let Some(task_handle) = self.task_handle.as_mut() {
                    if task_handle.target.peers.len() == 1
                        && task_handle.target.peers[0].peer_id == close_peer_id
                    {
                        task_handle.task_handle.cancel();
                        info!("Cancel task handle because peer {} closed", close_peer_id);
                    } else {
                        task_handle
                            .target
                            .peers
                            .retain(|peers| peers.peer_id != close_peer_id);
                    }
                }
            }
        }
    }
}

impl EventHandler<Self, PeerNewBlock> for SyncService2 {
    fn handle_event(&mut self, msg: PeerNewBlock, ctx: &mut ServiceContext<SyncService2>) {
        let self_ref = ctx.self_ref();
        let connect_service = self.connector_service.clone();
        let block = msg.get_block();
        let id = block.id();
        let peer_id = msg.get_peer_id();
        let fut = async move {
            if let Err(e) = connect_service.send(ConnectBlockRequest { block }).await? {
                match e.downcast::<ConnectBlockError>() {
                    Ok(connect_error) => {
                        match connect_error {
                            ConnectBlockError::FutureBlock(_) => {
                                //TODO cache future block
                                self_ref.notify(CheckSyncEvent { force: false })?;
                            }
                            e => {
                                return Err(e.into());
                            }
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        };
        ctx.spawn(fut.then(move |result| async move {
            if let Err(e) = result {
                error!(
                    "Connect block {:?} from peer {:?} error: {:?}",
                    id, peer_id, e
                );
            }
        }));
    }
}

#[derive(Debug, Clone)]
pub struct StartSyncEvent {
    target: SyncTarget,
}

impl EventHandler<Self, StartSyncEvent> for SyncService2 {
    fn handle_event(&mut self, msg: StartSyncEvent, ctx: &mut ServiceContext<SyncService2>) {
        let target_block_header = msg.target.block_header.clone();
        if let Err(e) = self.start_sync_task(msg.target, ctx) {
            error!(
                "Start sync task error: {:?}, target: {:?}",
                e, target_block_header
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckSyncEvent {
    force: bool,
}

impl EventHandler<Self, CheckSyncEvent> for SyncService2 {
    fn handle_event(&mut self, msg: CheckSyncEvent, ctx: &mut ServiceContext<SyncService2>) {
        if let Err(e) = self.check_sync(msg.force, ctx) {
            error!("Check sync error: {:?}", e);
        };
    }
}
