// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub type NodeManagerClient = jsonrpsee::async_client::Client;
use crate::FutureResult;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ServiceInfo, ServiceStatus};
use std::sync::Arc;
pub trait NodeManagerApi {
    fn list_service(&self) -> FutureResult<Vec<ServiceInfo>>;
    fn stop_service(&self, service_name: String) -> FutureResult<()>;
    fn start_service(&self, service_name: String) -> FutureResult<()>;
    fn check_service(&self, service_name: String) -> FutureResult<ServiceStatus>;
    fn shutdown_system(&self) -> FutureResult<()>;
    fn reset_to_block(&self, block_hash: HashValue) -> FutureResult<()>;

    /// Re execute the block of `block_id` for fix database
    fn re_execute_block(&self, block_hash: HashValue) -> FutureResult<()>;

    // /// Delete block data in [start_number, end_number)
    // #[rpc(name = "node_manager.delete_block_range")]
    // fn delete_block_range(
    //     &self,
    //     start_block_number: u64,
    //     end_block_number: u64,
    // ) -> FutureResult<()>;

    /// Delete block of block_id
    fn delete_block(&self, block_hash: HashValue) -> FutureResult<()>;

    /// Delete failed block of block_id from failed block database
    fn delete_failed_block(&self, block_hash: HashValue) -> FutureResult<()>;
}

/// Build jsonrpsee methods from legacy `NodeManagerApi`.
pub fn node_manager_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: NodeManagerApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_async_method("node_manager.list_service", |_, api, _| async move {
        api.list_service().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("node_manager.stop_service", |params, api, _| async move {
        let service_name: String = params.one()?;
        api.stop_service(service_name)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("node_manager.start_service", |params, api, _| async move {
        let service_name: String = params.one()?;
        api.start_service(service_name)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("node_manager.check_service", |params, api, _| async move {
        let service_name: String = params.one()?;
        api.check_service(service_name)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("node_manager.shutdown_system", |_, api, _| async move {
        api.shutdown_system().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("node_manager.reset_to_block", |params, api, _| async move {
        let block_hash: HashValue = params.one()?;
        api.reset_to_block(block_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("node_manager.re_execute_block", |params, api, _| async move {
        let block_hash: HashValue = params.one()?;
        api.re_execute_block(block_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("node_manager.delete_block", |params, api, _| async move {
        let block_hash: HashValue = params.one()?;
        api.delete_block(block_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("node_manager.delete_failed_block", |params, api, _| async move {
        let block_hash: HashValue = params.one()?;
        api.delete_failed_block(block_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}
