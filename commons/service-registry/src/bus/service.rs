// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::bus::sys_bus::SysBus;
use crate::bus::{Broadcast, Channel, Oneshot, Subscription};
use crate::{ActorService, ServiceContext, ServiceHandler};
use anyhow::Result;
use futures::channel::{mpsc, oneshot};
use std::fmt::Debug;

#[derive(Default)]
pub struct BusService {
    bus: SysBus,
}

impl ActorService for BusService {}

impl<M> ServiceHandler<Self, Subscription<M>> for BusService
where
    M: Send + Clone + Debug,
{
    fn handle(&mut self, msg: Subscription<M>, _ctx: &mut ServiceContext<Self>) {
        self.bus.subscribe(msg.recipient);
    }
}

impl<M> ServiceHandler<Self, Broadcast<M>> for BusService
where
    M: Send + Clone + Debug,
{
    fn handle(&mut self, msg: Broadcast<M>, _ctx: &mut ServiceContext<Self>) {
        self.bus.broadcast(msg.msg);
    }
}

impl<M> ServiceHandler<Self, Channel<M>> for BusService
where
    M: Send + Clone + Debug,
{
    fn handle(
        &mut self,
        _msg: Channel<M>,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<mpsc::UnboundedReceiver<M>> {
        Ok(self.bus.channel())
    }
}

impl<M> ServiceHandler<Self, Oneshot<M>> for BusService
where
    M: Send + Clone + Debug,
{
    fn handle(
        &mut self,
        _msg: Oneshot<M>,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<oneshot::Receiver<M>> {
        Ok(self.bus.oneshot())
    }
}
