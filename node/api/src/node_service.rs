// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{NodeRequest, NodeResponse};
use crate::service_registry::ServiceInfo;
use actix::dev::ToEnvelope;
use actix::{Actor, Addr, Handler};
use anyhow::Result;

#[async_trait::async_trait]
pub trait NodeAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    async fn list_service(&self) -> Result<Vec<ServiceInfo>>;

    async fn stop_service(&self, service_name: String) -> Result<()>;

    async fn start_service(&self, service_name: String) -> Result<()>;

    async fn stop_system(self) -> Result<()>;
}

#[async_trait::async_trait]
impl<A> NodeAsyncService for Addr<A>
where
    A: Actor,
    A: Handler<NodeRequest>,
    A::Context: ToEnvelope<A, NodeRequest>,
    A: std::marker::Send,
{
    async fn list_service(&self) -> Result<Vec<ServiceInfo>> {
        let response = self.send(NodeRequest::ListService).await??;
        if let NodeResponse::Services(services) = response {
            Ok(services)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn stop_service(&self, service_name: String) -> Result<()> {
        let response = self.send(NodeRequest::StopService(service_name)).await??;
        if let NodeResponse::Result(result) = response {
            result
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn start_service(&self, service_name: String) -> Result<()> {
        let response = self.send(NodeRequest::StartService(service_name)).await??;
        if let NodeResponse::Result(result) = response {
            result
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn stop_system(self) -> Result<()> {
        self.try_send(NodeRequest::StopSystem)?;
        Ok(())
    }
}
