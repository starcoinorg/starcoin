// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpc_core::Result;
use starcoin_rpc_api::status::StatusApi;

/// Re-export the API
pub use starcoin_rpc_api::status::*;

pub(crate) struct StatusRpcImpl {}

impl StatusRpcImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl StatusApi for StatusRpcImpl {
    fn status(&self) -> Result<bool> {
        //TODO check service status.
        Ok(true)
    }
}
