// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_logger::prelude::*;

mod account_rpc;
mod status_rpc;
mod txpool_rpc;

pub(crate) use self::status_rpc::StatusRpcImpl;
pub(crate) use self::txpool_rpc::TxPoolRpcImpl;

pub fn map_err(err: anyhow::Error) -> jsonrpc_core::Error {
    //TODO error convert.
    error!("rpc return internal_error for: {:?}", err);
    jsonrpc_core::Error::internal_error()
}
