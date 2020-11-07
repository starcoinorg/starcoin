// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use anyhow::Result;
use logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext};
use types::{
    node_status::NodeStatus,
    system_events::{NewHeadBlock, NodeStatusChangeEvent},
};

/// HeadBlockPacemaker, only generate block when new HeadBlock publish.
#[derive(Default)]
pub struct HeadBlockPacemaker {
    node_status: Option<NodeStatus>,
}

impl HeadBlockPacemaker {
    pub fn send_event(&mut self, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new(true));
    }
}

impl ActorService for HeadBlockPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<NodeStatusChangeEvent>();
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NodeStatusChangeEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for HeadBlockPacemaker {
    fn handle_event(&mut self, _msg: NewHeadBlock, ctx: &mut ServiceContext<HeadBlockPacemaker>) {
        if let Some(node_status) = self.node_status.as_ref() {
            if node_status.is_synced() {
                self.send_event(ctx)
            } else {
                debug!("Node has not synchronized, do not fire generate block event by HeadBlockPacemaker .");
            }
        }
    }
}

impl EventHandler<Self, NodeStatusChangeEvent> for HeadBlockPacemaker {
    fn handle_event(&mut self, msg: NodeStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.node_status = Some(msg.0);
    }
}
