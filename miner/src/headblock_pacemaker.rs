// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::prelude::*;

use futures::channel::mpsc;

use bus::{BusActor, Subscription};
use std::time::Duration;
use types::system_events::SystemEvents;

/// HeadBlockPacemaker, only generate block when new HeadBlock publish.
pub(crate) struct HeadBlockPacemaker {
    bus: Addr<BusActor>,
    sender: mpsc::Sender<GenerateBlockEvent>,
}

impl HeadBlockPacemaker {
    pub fn new(bus: Addr<BusActor>, sender: mpsc::Sender<GenerateBlockEvent>) -> Self {
        Self { bus, sender }
    }

    pub fn send_event(&mut self) {
        //TODO handle result.
        self.sender.try_send(GenerateBlockEvent {});
    }
}

impl Actor for HeadBlockPacemaker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<SystemEvents>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        info!("head block pacemaker started.");
    }
}

impl Handler<SystemEvents> for HeadBlockPacemaker {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SystemEvents::NewHeadBlock(_block) => self.send_event(),
            _ => {}
        }
    }
}
