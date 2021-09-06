// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_crypto::HashValue;
use starcoin_node_api::node_service::NodeAsyncService;
use starcoin_rpc_api::node_manager::NodeManagerApi;
use starcoin_rpc_api::FutureResult;
use starcoin_service_registry::{ServiceInfo, ServiceStatus};

pub struct NodeManagerRpcImpl<S>
where
    S: NodeAsyncService + 'static,
{
    service: S,
}

impl<S> NodeManagerRpcImpl<S>
where
    S: NodeAsyncService,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> NodeManagerApi for NodeManagerRpcImpl<S>
where
    S: NodeAsyncService,
{
    fn list_service(&self) -> FutureResult<Vec<ServiceInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.list_service().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn stop_service(&self, service_name: String) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service.stop_service(service_name).await?;
            Ok(())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn start_service(&self, service_name: String) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service.start_service(service_name).await?;
            Ok(())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn check_service(&self, service_name: String) -> FutureResult<ServiceStatus> {
        let service = self.service.clone();
        let fut = async move { service.check_service(service_name).await }.map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn shutdown_system(&self) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service.shutdown_system().await?;
            Ok(())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn reset_to_block(&self, block_hash: HashValue) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service.reset_node(block_hash).await?;
            Ok(())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn re_execute_block(&self, block_hash: HashValue) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service.re_execute_block(block_hash).await?;
            Ok(())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn delete_block(&self, block_id: HashValue) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service.delete_block(block_id).await?;
            Ok(())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn delete_failed_block(&self, block_id: HashValue) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service.delete_failed_block(block_id).await?;
            Ok(())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }
}
