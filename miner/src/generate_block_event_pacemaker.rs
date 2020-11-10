// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use anyhow::Result;
use logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext};
use tx_relay::PropagateNewTransactions;
use types::{
    sync_status::SyncStatus,
    system_events::{NewHeadBlock, SyncStatusChangeEvent},
};

#[derive(Default)]
pub struct GenerateBlockEventPacemaker {
    sync_status: Option<SyncStatus>,
}

impl GenerateBlockEventPacemaker {
    pub fn send_event(&mut self, force: bool, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new(force));
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
        ctx.subscribe::<PropagateNewTransactions>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        ctx.unsubscribe::<PropagateNewTransactions>();
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for GenerateBlockEventPacemaker {
    fn handle_event(
        &mut self,
        _msg: NewHeadBlock,
        ctx: &mut ServiceContext<GenerateBlockEventPacemaker>,
    ) {
        if self.is_synced() {
            self.send_event(true, ctx)
        } else {
            debug!("[pacemaker] Ignore NewHeadBlock event because the node has not been synchronized yet.")
        }
    }
}

impl EventHandler<Self, PropagateNewTransactions> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: PropagateNewTransactions, ctx: &mut ServiceContext<Self>) {
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
