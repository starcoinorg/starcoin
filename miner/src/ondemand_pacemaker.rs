// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::Addr;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use tx_relay::PropagateNewTransactions;

/// On-demand generate block, only generate block when new transaction add to tx-pool.
pub struct OndemandPacemaker {
    bus: Addr<BusActor>,
}

impl OndemandPacemaker {
    pub fn new(bus: Addr<BusActor>) -> Self {
        Self { bus }
    }

    pub fn send_event(&mut self) {
        let bus = self.bus.clone();
        if let Err(e) = bus.try_send(Broadcast::new(GenerateBlockEvent::new(false))) {
            error!("OndemandPacemaker send event error:  : {:?}", e);
        }
    }
}

impl ServiceFactory<Self> for OndemandPacemaker {
    fn create(ctx: &mut ServiceContext<OndemandPacemaker>) -> Result<OndemandPacemaker> {
        Ok(Self::new(ctx.get_shared::<Addr<BusActor>>()?))
    }
}

impl ActorService for OndemandPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) {
        ctx.subscribe::<PropagateNewTransactions>();
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) {
        ctx.unsubscribe::<PropagateNewTransactions>();
    }
}

impl EventHandler<Self, PropagateNewTransactions> for OndemandPacemaker {
    fn handle_event(
        &mut self,
        _msg: PropagateNewTransactions,
        _ctx: &mut ServiceContext<OndemandPacemaker>,
    ) {
        self.send_event()
    }
}
