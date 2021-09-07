// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as NodeManagerClient;
use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ServiceInfo, ServiceStatus};

#[rpc(client, server, schema)]
pub trait NodeManagerApi {
    #[rpc(name = "node_manager.list_service")]
    fn list_service(&self) -> FutureResult<Vec<ServiceInfo>>;

    #[rpc(name = "node_manager.stop_service")]
    fn stop_service(&self, service_name: String) -> FutureResult<()>;

    #[rpc(name = "node_manager.start_service")]
    fn start_service(&self, service_name: String) -> FutureResult<()>;

    #[rpc(name = "node_manager.check_service")]
    fn check_service(&self, service_name: String) -> FutureResult<ServiceStatus>;

    #[rpc(name = "node_manager.shutdown_system")]
    fn shutdown_system(&self) -> FutureResult<()>;
    #[rpc(name = "node_manager.reset_to_block")]
    fn reset_to_block(&self, block_hash: HashValue) -> FutureResult<()>;

    /// Re execute the block of `block_id` for fix database
    #[rpc(name = "node_manager.re_execute_block")]
    fn re_execute_block(&self, block_hash: HashValue) -> FutureResult<()>;

    // /// Delete block data in [start_number, end_number)
    // #[rpc(name = "node_manager.delete_block_range")]
    // fn delete_block_range(
    //     &self,
    //     start_block_number: u64,
    //     end_block_number: u64,
    // ) -> FutureResult<()>;

    /// Delete block of block_id
    #[rpc(name = "node_manager.delete_block")]
    fn delete_block(&self, block_hash: HashValue) -> FutureResult<()>;

    /// Delete failed block of block_id from failed block database
    #[rpc(name = "node_manager.delete_failed_block")]
    fn delete_failed_block(&self, block_hash: HashValue) -> FutureResult<()>;
}
#[test]
fn test() {
    let schema = rpc_impl_NodeManagerApi::gen_client::Client::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
