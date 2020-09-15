// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_service_registry::{ActorService, ServiceContext, ServiceHandler, ServiceRequest};

#[derive(Default, Clone)]
pub struct PanicService {
    counter: u64,
}

impl ActorService for PanicService {}

#[derive(Debug)]
pub struct PingRequest;

impl ServiceRequest for PingRequest {
    type Response = u64;
}

impl ServiceHandler<Self, PingRequest> for PanicService {
    fn handle(&mut self, _msg: PingRequest, _ctx: &mut ServiceContext<PanicService>) -> u64 {
        self.counter += 1;
        self.counter
    }
}

#[derive(Debug)]
pub struct PanicRequest;

impl ServiceRequest for PanicRequest {
    type Response = ();
}

impl ServiceHandler<Self, PanicRequest> for PanicService {
    fn handle(&mut self, _msg: PanicRequest, _ctx: &mut ServiceContext<PanicService>) {
        panic!("Panic by request.");
    }
}
