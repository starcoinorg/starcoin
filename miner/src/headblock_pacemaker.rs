// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::{
    Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, ContextFutureSpawner, Handler,
    WrapFuture,
};
use anyhow::Result;
use bus::{Broadcast, BusActor, Subscription};
use logger::prelude::*;
use starcoin_node_api::service_registry::{ServiceRegistry, SystemService};
use types::system_events::{ActorStop, NewHeadBlock};

/// HeadBlockPacemaker, only generate block when new HeadBlock publish.
pub struct HeadBlockPacemaker {
    bus: Addr<BusActor>,
}

impl HeadBlockPacemaker {
    pub fn new(service_registry: &ServiceRegistry) -> Result<Self> {
        Ok(Self {
            bus: service_registry.bus(),
        })
    }

    pub fn send_event(&mut self) {
        let bus = self.bus.clone();
        if let Err(e) = bus.try_send(Broadcast::new(GenerateBlockEvent::new(true))) {
            error!("HeadBlockPacemaker send event error:  : {:?}", e);
        }
    }
}

impl Actor for HeadBlockPacemaker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<NewHeadBlock>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        info!("HeadBlockPacemaker started");
        info!("{}", "Fire first GenerateBlock event");
        self.send_event();
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("HeadBlockPacemaker stopped");
    }
}

impl SystemService for HeadBlockPacemaker {}

impl Handler<ActorStop> for HeadBlockPacemaker {
    type Result = ();

    fn handle(&mut self, _msg: ActorStop, ctx: &mut Context<Self>) -> Self::Result {
        ctx.stop()
    }
}

impl Handler<NewHeadBlock> for HeadBlockPacemaker {
    type Result = ();

    fn handle(&mut self, _msg: NewHeadBlock, _ctx: &mut Self::Context) -> Self::Result {
        self.send_event()
    }
}
