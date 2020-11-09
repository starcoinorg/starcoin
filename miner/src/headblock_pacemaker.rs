// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use anyhow::Result;
use logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext};
use types::{
    sync_status::SyncStatus,
    system_events::{NewHeadBlock, SyncStatusChangeEvent},
};

/// HeadBlockPacemaker, only generate block when new HeadBlock publish.
#[derive(Default)]
pub struct HeadBlockPacemaker {
    sync_status: Option<SyncStatus>,
}

impl HeadBlockPacemaker {
    pub fn send_event(&mut self, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new(true));
    }
}

impl ActorService for HeadBlockPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for HeadBlockPacemaker {
    fn handle_event(&mut self, _msg: NewHeadBlock, ctx: &mut ServiceContext<HeadBlockPacemaker>) {
        if let Some(sync_status) = self.sync_status.as_ref() {
            if sync_status.is_synced() {
                self.send_event(ctx)
            } else {
                debug!("Node has not synchronized, do not fire generate block event by HeadBlockPacemaker .");
            }
        }
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for HeadBlockPacemaker {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.sync_status = Some(msg.0);
    }
}
