// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod delegates;
pub mod server;

use anyhow::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetRpcError {
    /// Message
    pub message: String,
}

impl NetRpcError {
    fn new(msg: String) -> Self {
        Self { message: msg }
    }
}

impl std::error::Error for NetRpcError {}

impl std::fmt::Display for NetRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Rpc error: {}", self.message)
    }
}

pub type Result<T, E = NetRpcError> = core::result::Result<T, E>;

impl From<anyhow::Error> for NetRpcError {
    fn from(any_err: Error) -> Self {
        NetRpcError::new(any_err.to_string())
    }
}
