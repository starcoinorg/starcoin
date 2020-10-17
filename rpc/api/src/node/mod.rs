// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as NodeClient;
use crate::FutureResult;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};
use starcoin_config::ChainNetwork;
use starcoin_types::peer_info::PeerInfo;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node self peer info
    pub peer_info: PeerInfo,
    pub self_address: String,
    pub net: ChainNetwork,
    pub consensus: ConsensusStrategy,
    pub now: u64,
}

impl NodeInfo {
    pub fn new(peer_info: PeerInfo, self_address: String, net: ChainNetwork) -> Self {
        let consensus = net.consensus();
        let now = net.time_service().now_secs();
        Self {
            peer_info,
            self_address,
            net,
            consensus,
            now,
        }
    }
}

#[rpc]
pub trait NodeApi {
    /// Get node run status, just for api available check.
    #[rpc(name = "node.status")]
    fn status(&self) -> Result<bool>;

    /// Get node self info.
    #[rpc(name = "node.info")]
    fn info(&self) -> FutureResult<NodeInfo>;

    /// Get current node connect peers.
    #[rpc(name = "node.peers")]
    fn peers(&self) -> FutureResult<Vec<PeerInfo>>;

    #[rpc(name = "node.metrics")]
    fn metrics(&self) -> Result<HashMap<String, String>>;
}
