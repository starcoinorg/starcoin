// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::bus::sys_bus::SysBus;
use crate::bus::{
    BroadcastRequest, ChannelRequest, OneshotRequest, SubscribeRequest, UnsubscribeRequest,
};
use crate::{ActorService, ServiceContext, ServiceHandler};
use anyhow::Result;
use futures::channel::{mpsc, oneshot};
use std::fmt::Debug;

#[derive(Default)]
pub struct BusService {
    bus: SysBus,
}

impl ActorService for BusService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.set_mailbox_capacity(1024);
        Ok(())
    }
}

impl<M> ServiceHandler<Self, SubscribeRequest<M>> for BusService
where
    M: Send + Clone + Debug,
{
    fn handle(&mut self, msg: SubscribeRequest<M>, _ctx: &mut ServiceContext<Self>) {
        self.bus.subscribe(msg.notifier);
    }
}

impl<M> ServiceHandler<Self, UnsubscribeRequest<M>> for BusService
where
    M: Send + Clone + Debug,
{
    fn handle(&mut self, msg: UnsubscribeRequest<M>, _ctx: &mut ServiceContext<Self>) {
        self.bus.unsubscribe::<M>(msg.target_service);
    }
}

impl<M> ServiceHandler<Self, BroadcastRequest<M>> for BusService
where
    M: Send + Clone + Debug,
{
    fn handle(&mut self, msg: BroadcastRequest<M>, _ctx: &mut ServiceContext<Self>) {
        self.bus.broadcast(msg.msg);
    }
}

impl<M> ServiceHandler<Self, ChannelRequest<M>> for BusService
where
    M: Send + Clone + Debug,
{
    fn handle(
        &mut self,
        _msg: ChannelRequest<M>,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<mpsc::UnboundedReceiver<M>> {
        Ok(self.bus.channel())
    }
}

impl<M> ServiceHandler<Self, OneshotRequest<M>> for BusService
where
    M: Send + Clone + Debug,
{
    fn handle(
        &mut self,
        _msg: OneshotRequest<M>,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<oneshot::Receiver<M>> {
        Ok(self.bus.oneshot())
    }
}
