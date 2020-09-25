// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use anyhow::Result;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext};
use tx_relay::PropagateNewTransactions;

/// On-demand generate block, only generate block when new transaction add to tx-pool.
#[derive(Default)]
pub struct OndemandPacemaker {}

impl OndemandPacemaker {
    pub fn send_event(&self, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new(false));
    }
}

impl ActorService for OndemandPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<PropagateNewTransactions>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<PropagateNewTransactions>();
        Ok(())
    }
}

impl EventHandler<Self, PropagateNewTransactions> for OndemandPacemaker {
    fn handle_event(
        &mut self,
        _msg: PropagateNewTransactions,
        ctx: &mut ServiceContext<OndemandPacemaker>,
    ) {
        self.send_event(ctx)
    }
}
