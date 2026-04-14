// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub(crate) fn map_err(err: anyhow::Error) -> anyhow::Error {
    err
}

pub(crate) fn map_jsonrpc_err(err: anyhow::Error) -> jsonrpsee::types::ErrorObjectOwned {
    jsonrpsee::types::ErrorObjectOwned::owned(
        jsonrpsee::types::error::INTERNAL_ERROR_CODE,
        err.to_string(),
        None::<()>,
    )
}

pub mod account_rpc;
pub mod contract_rpc;
mod helpers;
pub mod state_rpc;
