// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub type NodeClient = jsonrpsee::async_client::Client;
use crate::types::PeerInfoView;
use crate::FutureResult;
use anyhow::Result;
use jsonrpsee::{
    core::RegisterMethodError,
    Methods, RpcModule,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_config::ChainNetworkID;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::collections::HashMap;
use std::sync::Arc;

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
pub trait NodeApi {
    /// Get node run status, just for api available check.
    fn status(&self) -> Result<bool>;

    /// Get node self info.
    fn info(&self) -> FutureResult<NodeInfo>;

    /// Get current node connect peers.
    fn peers(&self) -> FutureResult<Vec<PeerInfoView>>;
    fn metrics(&self) -> Result<HashMap<String, String>>;
}

/// Build jsonrpsee methods from legacy `NodeApi`.
///
/// This helper allows migration to jsonrpsee runtime while keeping the current
/// `NodeApi` interface and implementations unchanged.
pub fn node_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: NodeApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_method("node.status", |_, api, _| api.status().map_err(crate::map_jsonrpc_err))?;

    module.register_async_method("node.info", |_, api, _| async move {
        api.info().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("node.peers", |_, api, _| async move {
        api.peers().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_method("node.metrics", |_, api, _| api.metrics().map_err(crate::map_jsonrpc_err))?;

    Ok(module.into())
}
