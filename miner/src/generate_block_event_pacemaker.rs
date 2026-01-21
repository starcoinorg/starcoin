// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{GenerateBlockEvent, NewHeaderChannel};
use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_txpool_api::PropagateTransactions;
use starcoin_types::{
    sync_status::SyncStatus,
    system_events::{NewHeadBlock, SyncStatusChangeEvent},
};
use std::sync::Arc;

pub struct GenerateBlockEventPacemaker {
    config: Arc<NodeConfig>,
    sync_status: Option<SyncStatus>,
    new_header_channel: NewHeaderChannel,
}

impl ServiceFactory<Self> for GenerateBlockEventPacemaker {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        Ok(Self {
            config: ctx.get_shared::<Arc<NodeConfig>>()?,
            sync_status: None,
            new_header_channel: ctx.get_shared::<NewHeaderChannel>()?,
        })
    }
}

impl GenerateBlockEventPacemaker {
    pub fn send_event(&mut self, force: bool, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new_break(force));
    }

    fn send_head_and_kick(&mut self, ctx: &mut ServiceContext<Self>) {
        let Some(status) = self.sync_status.as_ref() else {
            return;
        };
        let _consume = self
            .new_header_channel
            .new_header_receiver
            .try_iter()
            .count();
        match self
            .new_header_channel
            .new_header_sender
            .send(Arc::new(status.chain_status().head().clone()))
        {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to send header to new header channel: {:?}", e);
                return;
            }
        }
        // initial kick once node is synchronized
        self.send_event(true, ctx);
    }

    fn init_sync_status_from_shared(&mut self, ctx: &mut ServiceContext<Self>) {
        if self.sync_status.is_some() {
            return;
        }
        match ctx.get_shared_opt::<SyncStatus>() {
            Ok(Some(status)) => {
                self.sync_status = Some(status);
                if self.is_synced() {
                    self.send_head_and_kick(ctx);
                }
            }
            Ok(None) => {}
            Err(e) => warn!("[pacemaker] Get shared SyncStatus failed: {:?}", e),
        }
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
        //if mint empty block is disabled, trigger mint event for on demand mint (Dev)
        if self.config.miner.is_disable_mint_empty_block() {
            ctx.subscribe::<PropagateTransactions>();
        }
        self.init_sync_status_from_shared(ctx);
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        if self.config.miner.is_disable_mint_empty_block() {
            ctx.unsubscribe::<PropagateTransactions>();
        }
        Ok(())
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
        let was_synced = self.is_synced();
        self.sync_status = Some(msg.0);
        let now_synced = self.is_synced();

        if now_synced && !was_synced {
            self.send_head_and_kick(ctx);
        }
    }
}

impl EventHandler<Self, NewHeadBlock> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: NewHeadBlock, ctx: &mut ServiceContext<Self>) {
        if self.is_synced() {
            self.send_event(true, ctx);
        }
    }
}
