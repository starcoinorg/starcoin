// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{GenerateBlockEvent, NewHeaderChannel};
use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{BlockStore, Storage};
use starcoin_txpool_api::PropagateTransactions;
use starcoin_types::{
    startup_info::StartupInfo,
    sync_status::SyncStatus,
    system_events::{
        NewDagBlock, NewDagBlockFromPeer, NewHeadBlock, SyncStatusChangeEvent, SystemStarted,
    },
};
use std::sync::Arc;

pub struct GenerateBlockEventPacemaker {
    config: Arc<NodeConfig>,
    sync_status: Option<SyncStatus>,
}

impl ServiceFactory<Self> for GenerateBlockEventPacemaker {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        Ok(Self {
            config: ctx.get_shared::<Arc<NodeConfig>>()?,
            sync_status: None,
        })
    }
}

impl GenerateBlockEventPacemaker {
    pub fn send_event(&mut self, force: bool, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new_break(force));
    }

    pub fn is_synced(&self) -> bool {
        match &self.sync_status {
            Some(status) => match status.sync_status() {
                starcoin_types::sync_status::SyncState::Prepare => false,
                starcoin_types::sync_status::SyncState::Synchronizing {
                    target: _,
                    total_difficulty: _,
                } => false,
                starcoin_types::sync_status::SyncState::Synchronized => true,
            },
            None => false,
        }
    }
}

impl ActorService for GenerateBlockEventPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<NewHeadBlock>();
        ctx.subscribe::<NewDagBlockFromPeer>();
        ctx.subscribe::<NewDagBlock>();
        ctx.subscribe::<SystemStarted>();
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
        ctx.unsubscribe::<NewDagBlock>();
        ctx.unsubscribe::<SystemStarted>();
        if self.config.miner.is_disable_mint_empty_block() {
            ctx.unsubscribe::<PropagateTransactions>();
        }
        Ok(())
    }
}

impl EventHandler<Self, SystemStarted> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: SystemStarted, ctx: &mut ServiceContext<Self>) {
        let config = ctx
            .get_shared::<Arc<NodeConfig>>()
            .expect("config should exist");
        if config.miner.is_disable_mint_empty_block()
            || config.net().is_dev()
            || config.net().is_dag_test()
            || config.net().is_test()
        {
            return;
        }
        let channel = ctx
            .get_shared::<NewHeaderChannel>()
            .expect("new header channel should exist");
        let startup_info = ctx
            .get_shared::<StartupInfo>()
            .expect("startup info should exist");
        let storage = ctx
            .get_shared::<Arc<Storage>>()
            .expect("storage should exist");
        let header = storage
            .get_block_header_by_hash(startup_info.main)
            .expect("failed to get the header for startup info")
            .expect("the block in storage should exist");
        match channel.new_header_sender.send(Arc::new(header)) {
            Ok(_) => (),
            Err(e) => panic!("Failed to send header to new header channel: {:?}", e),
        }
        self.send_event(true, ctx);
    }
}

impl EventHandler<Self, NewDagBlock> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: NewDagBlock, ctx: &mut ServiceContext<Self>) {
        if self.is_synced() {
            self.send_event(false, ctx)
        } else {
            debug!("[pacemaker] Ignore NewDagBlock event because the node has not been synchronized yet.")
        }
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
        // let is_synced = msg.0.is_synced();
        self.sync_status = Some(msg.0);
        if self.is_synced() {
            self.send_event(true, ctx);
        }
    }
}

impl EventHandler<Self, NewDagBlockFromPeer> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: NewDagBlockFromPeer, ctx: &mut ServiceContext<Self>) {
        self.send_event(false, ctx);
    }
}
