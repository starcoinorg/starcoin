// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::{BoxFuture, Error};
use jsonrpsee::types::ErrorObjectOwned;

pub type FutureResult<T> = BoxFuture<Result<T, Error>>;
pub use jsonrpc_core::Params;

pub(crate) fn map_jsonrpc_err(err: jsonrpc_core::Error) -> ErrorObjectOwned {
    let code = i32::try_from(err.code.code()).unwrap_or(jsonrpsee::types::error::INTERNAL_ERROR_CODE);
    let data = err
        .data
        .as_ref()
        .and_then(|v| serde_json::value::to_raw_value(v).ok());
    ErrorObjectOwned::owned(code, err.message, data)
}

pub mod account;
pub mod chain;
pub mod contract_api;
pub mod debug;
pub mod errors;
pub mod metadata;
pub mod miner;
pub mod network_manager;
pub mod node;
pub mod node_manager;
pub mod pubsub;
pub mod service;
pub mod state;
pub mod sync_manager;
#[cfg(test)]
mod tests;
pub mod txpool;
pub mod types;

pub mod multi_types;
