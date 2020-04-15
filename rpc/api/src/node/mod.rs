// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

pub use self::gen_client::Client as NodeClient;
use crate::FutureResult;
use serde::{Deserialize, Serialize};
use starcoin_types::peer_info::PeerInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node self peer info
    pub peer_info: PeerInfo,
    pub self_address: String,
    //TODO add more node info
}

impl NodeInfo {
    pub fn new(peer_info: PeerInfo, self_address: String) -> Self {
        Self {
            peer_info,
            self_address,
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
}
