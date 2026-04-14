// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::types::PeerInfoView;
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_config::ChainNetworkID;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NodeInfo {
    /// Node self peer info
    pub peer_info: PeerInfoView,
    pub self_address: String,
    pub net: ChainNetworkID,
    pub consensus: ConsensusStrategy,
    pub now_seconds: u64,
}

impl NodeInfo {
    pub fn new(
        peer_info: PeerInfoView,
        self_address: String,
        net: ChainNetworkID,
        consensus: ConsensusStrategy,
        now_seconds: u64,
    ) -> Self {
        Self {
            peer_info,
            self_address,
            net,
            consensus,
            now_seconds,
        }
    }
}
use starcoin_rpc_schema_derive::rpc_schema;

#[rpc_schema]
#[rpc(client, server, namespace = "node", namespace_separator = ".")]
pub trait NodeApi {
    /// Get node run status, just for api available check.
    #[method(name = "status")]
    fn status(&self) -> RpcResult<bool>;

    /// Get node self info.
    #[method(name = "info")]
    async fn info(&self) -> RpcResult<NodeInfo>;

    /// Get current node connect peers.
    #[method(name = "peers")]
    async fn peers(&self) -> RpcResult<Vec<PeerInfoView>>;

    #[method(name = "metrics")]
    fn metrics(&self) -> RpcResult<HashMap<String, String>>;
}

pub use NodeApiClient as NodeApiRpcClient;
pub use NodeApiServer as NodeApiRpcServer;

/// Build jsonrpsee methods from legacy `NodeApi`.
///
/// This helper allows migration to jsonrpsee runtime while keeping the current
/// `NodeApi` interface and implementations unchanged.
pub fn node_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: NodeApiServer + Send + Sync + 'static,
{
    Ok(NodeApiServer::into_rpc(api).into())
}
