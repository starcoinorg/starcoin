// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::prelude::*;

use futures::channel::mpsc;

use actix::clock::Duration;
use actix_rt::time::delay_for;
use bus::{BusActor, Subscription};
use logger::prelude::*;
use types::system_events::NewHeadBlock;

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
        if let Err(e) = self.sender.try_send(GenerateBlockEvent {}) {
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
        let mut sender = self.sender.clone();
        //TODO fire first GenerateBlock event when node is ready.
        Arbiter::spawn(async move {
            delay_for(Duration::from_secs(2)).await;
            info!("{}", "head block pacemaker started.");
            info!("{}", "Fire first GenerateBlock event");
            if let Err(e) = sender.try_send(GenerateBlockEvent {}) {
                error!("err : {:?}", e);
            }
        });
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
