// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use starcoin_crypto::HashValue;
use starcoin_rpc_schema_derive::rpc_schema;
use starcoin_service_registry::{ServiceInfo, ServiceStatus};

#[rpc_schema]
#[rpc(client, server, namespace = "node_manager", namespace_separator = ".")]
pub trait NodeManagerApi {
    #[method(name = "list_service")]
    async fn list_service(&self) -> RpcResult<Vec<ServiceInfo>>;

    #[method(name = "stop_service")]
    async fn stop_service(&self, service_name: String) -> RpcResult<()>;

    #[method(name = "start_service")]
    async fn start_service(&self, service_name: String) -> RpcResult<()>;

    #[method(name = "check_service")]
    async fn check_service(&self, service_name: String) -> RpcResult<ServiceStatus>;

    #[method(name = "shutdown_system")]
    async fn shutdown_system(&self) -> RpcResult<()>;

    #[method(name = "reset_to_block")]
    async fn reset_to_block(&self, block_hash: HashValue) -> RpcResult<()>;

    /// Re execute the block of `block_id` for fix database
    #[method(name = "re_execute_block")]
    async fn re_execute_block(&self, block_hash: HashValue) -> RpcResult<()>;

    // /// Delete block data in [start_number, end_number)
    // #[rpc(name = "node_manager.delete_block_range")]
    // fn delete_block_range(
    //     &self,
    //     start_block_number: u64,
    //     end_block_number: u64,
    // ) -> FutureResult<()>;

    /// Delete block of block_id
    #[method(name = "delete_block")]
    async fn delete_block(&self, block_hash: HashValue) -> RpcResult<()>;

    /// Delete failed block of block_id from failed block database
    #[method(name = "delete_failed_block")]
    async fn delete_failed_block(&self, block_hash: HashValue) -> RpcResult<()>;
}

pub use NodeManagerApiClient as NodeManagerApiRpcClient;
pub use NodeManagerApiServer as NodeManagerApiRpcServer;

/// Build jsonrpsee methods from legacy `NodeManagerApi`.
pub fn node_manager_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: NodeManagerApiServer + Send + Sync + 'static,
{
    Ok(NodeManagerApiServer::into_rpc(api).into())
}
