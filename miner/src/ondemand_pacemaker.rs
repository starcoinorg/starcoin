// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::prelude::*;

use futures::channel::mpsc;

use bus::{BusActor, Subscription};
use std::time::Duration;
use types::system_events::SystemEvents;

/// On-demand generate block, only generate block when new transaction add to tx-pool.
pub(crate) struct OndemandPacemaker {
    bus: Addr<BusActor>,
    sender: mpsc::Sender<GenerateBlockEvent>,
}

impl OndemandPacemaker {
    pub fn new(bus: Addr<BusActor>, sender: mpsc::Sender<GenerateBlockEvent>) -> Self {
        Self { bus, sender }
    }

    pub fn send_event(&mut self) {
        //TODO handle result.
        self.sender.try_send(GenerateBlockEvent {});
    }
}

impl Actor for OndemandPacemaker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<SystemEvents>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
    }
}

impl Handler<SystemEvents> for OndemandPacemaker {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SystemEvents::NewUserTransaction(_txn) => self.send_event(),
            _ => {}
        }
    }
}
