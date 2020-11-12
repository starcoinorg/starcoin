// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as NodeManagerClient;
use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_service_registry::{ServiceInfo, ServiceStatus};

#[rpc]
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
}
