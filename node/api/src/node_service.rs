// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{NodeRequest, NodeResponse};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{
    ActorService, ServiceHandler, ServiceInfo, ServiceRef, ServiceStatus,
};

pub trait NodeAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    fn list_service(&self) -> impl std::future::Future<Output = Result<Vec<ServiceInfo>>> + Send;

    fn stop_service(
        &self,
        service_name: String,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn start_service(
        &self,
        service_name: String,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn check_service(
        &self,
        service_name: String,
    ) -> impl std::future::Future<Output = Result<ServiceStatus>> + Send;

    fn start_pacemaker(&self) -> impl std::future::Future<Output = Result<()>> + Send;

    fn stop_pacemaker(&self) -> impl std::future::Future<Output = Result<()>> + Send;

    fn shutdown_system(&self) -> impl std::future::Future<Output = Result<()>> + Send;
    fn reset_node(
        &self,
        block_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
    fn re_execute_block(
        &self,
        block_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
    fn delete_block(
        &self,
        block_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
    fn delete_failed_block(
        &self,
        block_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl<A> NodeAsyncService for ServiceRef<A>
where
    A: ActorService,
    A: ServiceHandler<A, NodeRequest>,
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

    async fn check_service(&self, service_name: String) -> Result<ServiceStatus> {
        let response = self.send(NodeRequest::CheckService(service_name)).await??;
        if let NodeResponse::ServiceStatus(status) = response {
            Ok(status)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn start_pacemaker(&self) -> Result<()> {
        let response = self.send(NodeRequest::StartPacemaker).await??;
        if let NodeResponse::Result(result) = response {
            result
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn stop_pacemaker(&self) -> Result<()> {
        let response = self.send(NodeRequest::StopPacemaker).await??;
        if let NodeResponse::Result(result) = response {
            result
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn shutdown_system(&self) -> Result<()> {
        self.try_send(NodeRequest::ShutdownSystem)?;
        Ok(())
    }

    async fn reset_node(&self, block_hash: HashValue) -> Result<()> {
        let response = self.send(NodeRequest::ResetNode(block_hash)).await??;
        if let NodeResponse::AsyncResult(receiver) = response {
            return receiver.await?;
        }
        Ok(())
    }

    async fn re_execute_block(&self, block_hash: HashValue) -> Result<()> {
        let response = self.send(NodeRequest::ReExecuteBlock(block_hash)).await??;
        if let NodeResponse::AsyncResult(receiver) = response {
            return receiver.await?;
        }
        Ok(())
    }

    async fn delete_block(&self, block_hash: HashValue) -> Result<()> {
        self.try_send(NodeRequest::DeleteBlock(block_hash))?;
        Ok(())
    }

    async fn delete_failed_block(&self, block_hash: HashValue) -> Result<()> {
        self.try_send(NodeRequest::DeleteFailedBlock(block_hash))?;
        Ok(())
    }
}
