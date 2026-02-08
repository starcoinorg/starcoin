// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use futures::future::BoxFuture;
use jsonrpsee::types::ErrorObjectOwned;
pub use starcoin_vm2_abi_decoder::DecodedMoveValue;
pub type FutureResult<T> = BoxFuture<'static, anyhow::Result<T>>;

pub(crate) fn map_jsonrpc_err(err: anyhow::Error) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        jsonrpsee::types::error::INTERNAL_ERROR_CODE,
        err.to_string(),
        None::<()>,
    )
}

pub mod account_api;
pub mod block_info_view2;
pub mod contract_api;
pub mod state_api;
pub mod transaction_view2;
