// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use log::debug;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceHandler, ServiceRef, ServiceRequest,
};

#[async_trait::async_trait]
pub trait BroadcastProcessAsyncService {
    async fn get_msg_count(&self) -> Result<MsgCountResult>;
}

#[async_trait::async_trait]
impl BroadcastProcessAsyncService for ServiceRef<BroadcastProcessService> {
    async fn get_msg_count(&self) -> Result<MsgCountResult> {
        self.send(GetMsgCount).await
    }
}

#[derive(Default)]
pub struct BroadcastProcessService {
    b1_count: u64,
    b2_count: u64,
}

impl ActorService for BroadcastProcessService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<BMessage1>();
        ctx.subscribe::<BMessage2>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<BMessage1>();
        ctx.unsubscribe::<BMessage2>();
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BMessage1 {}

impl EventHandler<Self, BMessage1> for BroadcastProcessService {
    fn handle_event(&mut self, _msg: BMessage1, _ctx: &mut ServiceContext<Self>) {
        self.b1_count += 1;
        debug!("handle_broadcast b1, count: {}", self.b1_count);
    }
}

#[derive(Clone, Debug)]
pub struct BMessage2 {}

impl EventHandler<Self, BMessage2> for BroadcastProcessService {
    fn handle_event(&mut self, _msg: BMessage2, _ctx: &mut ServiceContext<Self>) {
        self.b2_count += 1;
        debug!("handle_broadcast b2, count: {}", self.b2_count);
    }
}

#[derive(Debug)]
pub struct GetMsgCount;

pub struct MsgCountResult {
    pub b1_count: u64,
    pub b2_count: u64,
}

impl ServiceRequest for GetMsgCount {
    type Response = MsgCountResult;
}

impl ServiceHandler<Self, GetMsgCount> for BroadcastProcessService {
    fn handle(&mut self, _msg: GetMsgCount, _ctx: &mut ServiceContext<Self>) -> MsgCountResult {
        MsgCountResult {
            b1_count: self.b1_count,
            b2_count: self.b2_count,
        }
    }
}
