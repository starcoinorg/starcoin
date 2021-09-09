// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as NodeClient;
use crate::types::PeerInfoView;
use crate::FutureResult;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
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

#[rpc(client, server, schema)]
pub trait NodeApi {
    /// Get node run status, just for api available check.
    #[rpc(name = "node.status")]
    fn status(&self) -> Result<bool>;

    /// Get node self info.
    #[rpc(name = "node.info")]
    fn info(&self) -> FutureResult<NodeInfo>;

    /// Get current node connect peers.
    #[rpc(name = "node.peers")]
    fn peers(&self) -> FutureResult<Vec<PeerInfoView>>;

    #[rpc(name = "node.metrics")]
    fn metrics(&self) -> Result<HashMap<String, String>>;
}
#[test]
fn test() {
    let schema = rpc_impl_NodeApi::gen_client::Client::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
