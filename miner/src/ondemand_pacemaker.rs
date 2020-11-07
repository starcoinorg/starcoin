// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use anyhow::Result;
use logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext};
use tx_relay::PropagateNewTransactions;
use types::{node_status::NodeStatus, system_events::NodeStatusChangeEvent};

/// On-demand generate block, only generate block when new transaction add to tx-pool.
#[derive(Default)]
pub struct OndemandPacemaker {
    node_status: Option<NodeStatus>,
}

impl OndemandPacemaker {
    pub fn send_event(&self, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new(false));
    }
}

impl ActorService for OndemandPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<NodeStatusChangeEvent>();
        ctx.subscribe::<PropagateNewTransactions>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NodeStatusChangeEvent>();
        ctx.unsubscribe::<PropagateNewTransactions>();
        Ok(())
    }
}

impl EventHandler<Self, NodeStatusChangeEvent> for OndemandPacemaker {
    fn handle_event(&mut self, msg: NodeStatusChangeEvent, ctx: &mut ServiceContext<Self>) {
        let is_synced = msg.0.is_synced();
        self.node_status = Some(msg.0);
        if is_synced {
            self.send_event(ctx);
        }
    }
}

impl EventHandler<Self, PropagateNewTransactions> for OndemandPacemaker {
    fn handle_event(&mut self, _msg: PropagateNewTransactions, ctx: &mut ServiceContext<Self>) {
        if let Some(node_status) = self.node_status.as_ref() {
            if node_status.is_nearly_synced() {
                self.send_event(ctx)
            } else {
                debug!("Node has not synchronized, do not fire generate block event by OndemandPacemaker.");
            }
        }
    }
}
