// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::WriteBlockChainService;
use crate::tasks::BlockConnectedEvent;
use anyhow::{format_err, Result};
use config::NodeConfig;
use logger::prelude::*;
use starcoin_chain_api::WriteableChainService;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::{BlockStore, Storage};
use starcoin_types::block::Block;
use starcoin_types::system_events::MinedBlock;
use std::sync::Arc;
use txpool::TxPoolService;

pub struct BlockConnectorService {
    chain_service: WriteBlockChainService<TxPoolService>,
}

impl BlockConnectorService {
    pub fn new(chain_service: WriteBlockChainService<TxPoolService>) -> Self {
        Self { chain_service }
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
        ctx.subscribe::<MinedBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
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

#[derive(Clone, Debug)]
pub struct ConnectBlockRequest {
    pub block: Block,
}

impl ServiceRequest for ConnectBlockRequest {
    type Response = Result<()>;
}

impl ServiceHandler<Self, ConnectBlockRequest> for BlockConnectorService {
    fn handle(&mut self, msg: ConnectBlockRequest, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        self.chain_service.try_connect(msg.block)
    }
}
