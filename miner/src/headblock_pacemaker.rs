// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use anyhow::Result;
use logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext};
use types::system_events::NewHeadBlock;

/// HeadBlockPacemaker, only generate block when new HeadBlock publish.
#[derive(Default)]
pub struct HeadBlockPacemaker {}

impl HeadBlockPacemaker {
    pub fn send_event(&mut self, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new(true));
    }
}

impl ActorService for HeadBlockPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<NewHeadBlock>();
        info!("{}", "Fire first GenerateBlock event");
        self.send_event(ctx);
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for HeadBlockPacemaker {
    fn handle_event(&mut self, _msg: NewHeadBlock, ctx: &mut ServiceContext<HeadBlockPacemaker>) {
        self.send_event(ctx)
    }
}
