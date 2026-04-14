// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::core::{async_trait, RpcResult};
use starcoin_crypto::HashValue;
use starcoin_node_api::node_service::NodeAsyncService;
use starcoin_rpc_api::node_manager::NodeManagerApiServer;
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

#[async_trait]
impl<S> NodeManagerApiServer for NodeManagerRpcImpl<S>
where
    S: NodeAsyncService,
{
    async fn list_service(&self) -> RpcResult<Vec<ServiceInfo>> {
        let service = self.service.clone();
        service
            .list_service()
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn stop_service(&self, service_name: String) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .stop_service(service_name)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(())
    }

    async fn start_service(&self, service_name: String) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .start_service(service_name)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(())
    }

    async fn check_service(&self, service_name: String) -> RpcResult<ServiceStatus> {
        let service = self.service.clone();
        service
            .check_service(service_name)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn shutdown_system(&self) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .shutdown_system()
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(())
    }

    async fn reset_to_block(&self, block_hash: HashValue) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .reset_node(block_hash)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(())
    }

    async fn re_execute_block(&self, block_hash: HashValue) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .re_execute_block(block_hash)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(())
    }

    async fn delete_block(&self, block_id: HashValue) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .delete_block(block_id)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(())
    }

    async fn delete_failed_block(&self, block_id: HashValue) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .delete_failed_block(block_id)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(())
    }
}
