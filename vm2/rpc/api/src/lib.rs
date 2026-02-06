// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::{BoxFuture, Error};
use jsonrpsee::types::ErrorObjectOwned;
pub use starcoin_vm2_abi_decoder::DecodedMoveValue;
pub type FutureResult<T> = BoxFuture<Result<T, Error>>;

pub(crate) fn map_jsonrpc_err(err: jsonrpc_core::Error) -> ErrorObjectOwned {
    let code = i32::try_from(err.code.code()).unwrap_or(jsonrpsee::types::error::INTERNAL_ERROR_CODE);
    let data = err
        .data
        .as_ref()
        .and_then(|v| serde_json::value::to_raw_value(v).ok());
    ErrorObjectOwned::owned(code, err.message, data)
}

pub mod account_api;
pub mod block_info_view2;
pub mod contract_api;
pub mod state_api;
pub mod transaction_view2;
