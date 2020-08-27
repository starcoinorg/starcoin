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
use tx_relay::PropagateNewTransactions;
use types::system_events::ActorStop;

/// On-demand generate block, only generate block when new transaction add to tx-pool.
pub struct OndemandPacemaker {
    bus: Addr<BusActor>,
}

impl OndemandPacemaker {
    pub fn new(service_registry: &ServiceRegistry) -> Result<Self> {
        Ok(Self {
            bus: service_registry.bus(),
        })
    }

    pub fn send_event(&mut self) {
        let bus = self.bus.clone();
        if let Err(e) = bus.try_send(Broadcast::new(GenerateBlockEvent::new(false))) {
            error!("OndemandPacemaker send event error:  : {:?}", e);
        }
    }
}

impl Actor for OndemandPacemaker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<PropagateNewTransactions>();
        self.bus
            .clone()
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        info!("Ondemand Pacemaker started.");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Ondemand Pacemaker stopped");
    }
}

impl SystemService for OndemandPacemaker {}

impl Handler<ActorStop> for OndemandPacemaker {
    type Result = ();

    fn handle(&mut self, _msg: ActorStop, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop()
    }
}

impl Handler<PropagateNewTransactions> for OndemandPacemaker {
    type Result = ();

    fn handle(&mut self, _msg: PropagateNewTransactions, ctx: &mut Self::Context) -> Self::Result {
        self.send_event()
    }
}
