// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::common::cal_service::{CalAsyncService, CalService};
use anyhow::Result;
use futures::executor::block_on;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef, ServiceRequest,
};

#[async_trait::async_trait]
pub trait AdvanceCalAsyncService {
    async fn batch_add(&self, values: Vec<u64>) -> Result<u64>;
}

#[async_trait::async_trait]
impl AdvanceCalAsyncService for ServiceRef<AdvanceCalService> {
    async fn batch_add(&self, values: Vec<u64>) -> Result<u64> {
        self.send(BatchAddRequest { values }).await
    }
}

pub struct AdvanceCalService {
    cal_service: ServiceRef<CalService>,
}

#[derive(Debug)]
pub struct BatchAddRequest {
    values: Vec<u64>,
}

impl ServiceRequest for BatchAddRequest {
    type Response = u64;
}

impl ActorService for AdvanceCalService {}

impl ServiceFactory<Self> for AdvanceCalService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        Ok(Self {
            cal_service: ctx.service_ref::<CalService>()?.clone(),
        })
    }
}

impl ServiceHandler<Self, BatchAddRequest> for AdvanceCalService {
    fn handle(&mut self, msg: BatchAddRequest, _ctx: &mut ServiceContext<Self>) -> u64 {
        let mut result = 0;
        for v in msg.values {
            result = block_on(async { self.cal_service.add(v).await.unwrap() });
        }
        result
    }
}
