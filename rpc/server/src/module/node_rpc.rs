// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpc_core::Result;
use starcoin_rpc_api::node::NodeApi;

/// Re-export the API
pub use starcoin_rpc_api::node::*;

pub struct NodeRpcImpl {}

impl NodeRpcImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl NodeApi for NodeRpcImpl {
    fn status(&self) -> Result<bool> {
        //TODO check service status.
        Ok(true)
    }
}
