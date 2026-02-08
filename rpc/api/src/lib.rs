// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use futures::future::BoxFuture;
use jsonrpsee::types::ErrorObjectOwned;

pub type FutureResult<T> = BoxFuture<'static, anyhow::Result<T>>;
pub type Params = serde_json::Value;

pub(crate) fn map_jsonrpc_err(err: anyhow::Error) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        jsonrpsee::types::error::INTERNAL_ERROR_CODE,
        err.to_string(),
        None::<()>,
    )
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
