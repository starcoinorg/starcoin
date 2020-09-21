// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::Addr;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use types::system_events::NewHeadBlock;

/// HeadBlockPacemaker, only generate block when new HeadBlock publish.
pub struct HeadBlockPacemaker {
    bus: Addr<BusActor>,
}

impl HeadBlockPacemaker {
    pub fn new(bus: Addr<BusActor>) -> Self {
        Self { bus }
    }

    pub fn send_event(&mut self) {
        let bus = self.bus.clone();
        if let Err(e) = bus.try_send(Broadcast::new(GenerateBlockEvent::new(true))) {
            error!("HeadBlockPacemaker send event error:  : {:?}", e);
        }
    }
}

impl ServiceFactory<Self> for HeadBlockPacemaker {
    fn create(ctx: &mut ServiceContext<HeadBlockPacemaker>) -> Result<HeadBlockPacemaker> {
        Ok(Self::new(ctx.get_shared::<Addr<BusActor>>()?))
    }
}

impl ActorService for HeadBlockPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<NewHeadBlock>();
        info!("{}", "Fire first GenerateBlock event");
        self.send_event();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for HeadBlockPacemaker {
    fn handle_event(&mut self, _msg: NewHeadBlock, _ctx: &mut ServiceContext<HeadBlockPacemaker>) {
        self.send_event()
    }
}
