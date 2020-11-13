// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::WriteBlockChainService;
use crate::sync2::{CheckSyncEvent, SyncService2};
use crate::tasks::BlockConnectedEvent;
use anyhow::{format_err, Result};
use config::NodeConfig;
use logger::prelude::*;
use starcoin_chain_api::{ConnectBlockError, WriteableChainService};
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::PeerNewBlock;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::{MinedBlock, SyncStatusChangeEvent};
use std::sync::Arc;
use txpool::TxPoolService;

pub struct BlockConnectorService {
    chain_service: WriteBlockChainService<TxPoolService>,
    sync_status: Option<SyncStatus>,
}

impl BlockConnectorService {
    pub fn new(chain_service: WriteBlockChainService<TxPoolService>) -> Self {
        Self {
            chain_service,
            sync_status: None,
        }
    }

    pub fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_synced(),
            None => false,
        }
    }
}

impl ServiceFactory<Self> for BlockConnectorService {
    fn create(ctx: &mut ServiceContext<BlockConnectorService>) -> Result<BlockConnectorService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let bus = ctx.bus_ref().clone();
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist."))?;
        let chain_service =
            WriteBlockChainService::new(config, startup_info, storage, txpool, bus, None)?;

        Ok(Self::new(chain_service))
    }
}

impl ActorService for BlockConnectorService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        //TODO figure out a more suitable value.
        ctx.set_mailbox_capacity(1024);
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<MinedBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<MinedBlock>();
        Ok(())
    }
}

impl EventHandler<Self, BlockConnectedEvent> for BlockConnectorService {
    fn handle_event(
        &mut self,
        msg: BlockConnectedEvent,
        _ctx: &mut ServiceContext<BlockConnectorService>,
    ) {
        //because this block has execute at sync task, so just try connect to select head chain.
        //TODO refactor connect and execute
        let block = msg.block;
        if let Err(e) = self.chain_service.try_connect(block) {
            error!("Process connected block error: {:?}", e);
        }
    }
}

impl EventHandler<Self, MinedBlock> for BlockConnectorService {
    fn handle_event(&mut self, msg: MinedBlock, _ctx: &mut ServiceContext<Self>) {
        let MinedBlock(new_block) = msg;
        let id = new_block.header().id();
        debug!("try connect mined block: {}", id);

        match self.chain_service.try_connect(new_block.as_ref().clone()) {
            Ok(_) => debug!("Process mined block {} success.", id),
            Err(e) => {
                warn!("Process mined block {} fail, error: {:?}", id, e);
            }
        }
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for BlockConnectorService {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.sync_status = Some(msg.0);
    }
}

impl EventHandler<Self, PeerNewBlock> for BlockConnectorService {
    fn handle_event(&mut self, msg: PeerNewBlock, ctx: &mut ServiceContext<Self>) {
        if !self.is_synced() {
            debug!("[connector] Ignore PeerNewBlock event because the node has not been synchronized yet.");
            return;
        }
        let peer_id = msg.get_peer_id();
        if let Err(e) = self.chain_service.try_connect(msg.get_block().clone()) {
            match e.downcast::<ConnectBlockError>() {
                Ok(connect_error) => {
                    match connect_error {
                        ConnectBlockError::FutureBlock(block) => {
                            //TODO cache future block
                            if let Ok(sync_service) = ctx.service_ref::<SyncService2>() {
                                info!(
                                    "BlockConnector try connect future block ({:?},{}), peer_id:{:?}, notify Sync service check sync.",
                                    block.id(),
                                    block.header().number,
                                    peer_id
                                );
                                let _ = sync_service.notify(CheckSyncEvent::new());
                            }
                        }
                        e => warn!("BlockConnector fail: {:?}, peer_id:{:?}", e, peer_id),
                    }
                }
                Err(e) => warn!("BlockConnector fail: {:?}, peer_id:{:?}", e, peer_id),
            }
        }
    }
}
