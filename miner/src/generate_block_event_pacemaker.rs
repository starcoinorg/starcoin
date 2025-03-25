// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_state_api::AccountStateReader;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{IntoSuper, Storage};
use starcoin_txpool_api::PropagateTransactions;
use starcoin_types::{
    sync_status::SyncStatus,
    system_events::{NewDagBlockFromPeer, NewHeadBlock, SyncStatusChangeEvent},
};
use std::sync::Arc;

pub struct GenerateBlockEventPacemaker {
    config: Arc<NodeConfig>,
    sync_status: Option<SyncStatus>,
    storage: Arc<Storage>,
}

impl ServiceFactory<Self> for GenerateBlockEventPacemaker {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        Ok(Self {
            config: ctx.get_shared::<Arc<NodeConfig>>()?,
            sync_status: None,
            storage: ctx.get_shared::<Arc<Storage>>()?,
        })
    }
}

impl GenerateBlockEventPacemaker {
    pub fn send_event(&mut self, force: bool, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new_break(force));
    }

    pub fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_synced(),
            None => false,
        }
    }
}

impl ActorService for GenerateBlockEventPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<NewHeadBlock>();
        ctx.subscribe::<NewDagBlockFromPeer>();
        //if mint empty block is disabled, trigger mint event for on demand mint (Dev)
        if self.config.miner.is_disable_mint_empty_block() {
            ctx.subscribe::<PropagateTransactions>();
        }
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        ctx.unsubscribe::<NewDagBlockFromPeer>();
        if self.config.miner.is_disable_mint_empty_block() {
            ctx.unsubscribe::<PropagateTransactions>();
        }
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: NewHeadBlock, ctx: &mut ServiceContext<Self>) {
        if self.is_synced() {
            self.send_event(true, ctx)
        } else {
            debug!("[pacemaker] Ignore NewHeadBlock event because the node has not been synchronized yet.")
        }
    }
}

impl EventHandler<Self, PropagateTransactions> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: PropagateTransactions, ctx: &mut ServiceContext<Self>) {
        if self.is_synced() {
            self.send_event(false, ctx)
        } else {
            debug!("[pacemaker] Ignore PropagateNewTransactions event because the node has not been synchronized yet.")
        }
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, ctx: &mut ServiceContext<Self>) {
        let is_synced = msg.0.is_synced();
        self.sync_status = Some(msg.0);
        if is_synced {
            self.send_event(false, ctx);
        }
    }
}

impl EventHandler<Self, NewDagBlockFromPeer> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, msg: NewDagBlockFromPeer, ctx: &mut ServiceContext<Self>) {
        let state_root = msg.executed_block.state_root();
        let chain_state =
            ChainStateDB::new(self.storage.clone().into_super_arc(), Some(state_root));
        let account_reader = AccountStateReader::new(&chain_state);
        let epoch_info = account_reader
            .get_epoch_info()
            .unwrap_or_else(|e| panic!("Epoch is none. error: {:?}", e));
        let total_uncles = epoch_info.epoch_data().uncles();
        let blocks = msg
            .executed_block
            .number()
            .saturating_sub(epoch_info.epoch().start_block_number());
        let uncle_rate = total_uncles * 1000 / blocks;
        let uncle_rate_in_confing = self
            .config
            .net()
            .genesis_config()
            .consensus_config
            .uncle_rate_target;
        info!("NewDagBlockFromPeer, epoch data: {:?}, blocks in this epoch: {:?}, uncle rate for now: {:?}, uncle rate in config: {:?}", epoch_info.epoch_data(), blocks, uncle_rate, uncle_rate_in_confing);
        if uncle_rate < uncle_rate_in_confing {
            self.send_event(false, ctx);
        }
    }
}
