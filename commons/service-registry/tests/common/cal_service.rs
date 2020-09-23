// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceHandler, ServiceRef, ServiceRequest,
};

#[async_trait::async_trait]
pub trait CalAsyncService {
    async fn add(&self, value: u64) -> Result<u64>;
    async fn sub(&self, value: u64) -> Result<u64>;
    async fn get(&self) -> Result<u64>;
}

#[async_trait::async_trait]
impl CalAsyncService for ServiceRef<CalService> {
    async fn add(&self, value: u64) -> Result<u64> {
        self.send(CalAddRequest { value }).await
    }

    async fn sub(&self, value: u64) -> Result<u64> {
        self.send(CalSubRequest { value }).await
    }

    async fn get(&self) -> Result<u64> {
        self.send(CalGetRequest).await
    }
}

#[derive(Default, Clone)]
pub struct CalService {
    value: u64,
}

impl CalService {
    pub fn add(&mut self, value: u64) -> u64 {
        self.value += value;
        self.value
    }
    pub fn sub(&mut self, value: u64) -> u64 {
        self.value -= value;
        self.value
    }
}

impl ActorService for CalService {}

#[derive(Clone, Debug, Default)]
pub struct CalAddRequest {
    pub value: u64,
}

impl ServiceRequest for CalAddRequest {
    type Response = u64;
}

impl ServiceHandler<Self, CalAddRequest> for CalService {
    fn handle(&mut self, msg: CalAddRequest, _ctx: &mut ServiceContext<Self>) -> u64 {
        self.add(msg.value)
    }
}

#[derive(Debug)]
pub struct CalSubRequest {
    pub value: u64,
}

impl ServiceRequest for CalSubRequest {
    type Response = u64;
}

impl ServiceHandler<Self, CalSubRequest> for CalService {
    fn handle(&mut self, msg: CalSubRequest, _ctx: &mut ServiceContext<Self>) -> u64 {
        self.sub(msg.value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct CalGetRequest;

impl ServiceRequest for CalGetRequest {
    type Response = u64;
}

impl ServiceHandler<Self, CalGetRequest> for CalService {
    fn handle(&mut self, _msg: CalGetRequest, _ctx: &mut ServiceContext<Self>) -> u64 {
        self.value
    }
}
