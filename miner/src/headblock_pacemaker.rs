// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::prelude::*;
use bus::{BusActor, Subscription};
use futures::channel::mpsc;
use logger::prelude::*;
use types::system_events::{NewHeadBlock, SystemStarted};

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
        if let Err(e) = self.sender.try_send(GenerateBlockEvent::new(true)) {
            error!("err : {:?}", e);
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

        let recipient = ctx.address().recipient::<SystemStarted>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        info!("HeadBlockPacemaker started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("HeadBlockPacemaker stopped");
    }
}

impl Handler<NewHeadBlock> for HeadBlockPacemaker {
    type Result = ();

    fn handle(&mut self, msg: NewHeadBlock, _ctx: &mut Self::Context) -> Self::Result {
        let NewHeadBlock(_block) = msg;
        self.send_event();
    }
}

impl Handler<SystemStarted> for HeadBlockPacemaker {
    type Result = ();

    fn handle(&mut self, _msg: SystemStarted, _ctx: &mut Self::Context) -> Self::Result {
        info!("{}", "Fire first GenerateBlock event");
        self.send_event();
    }
}
