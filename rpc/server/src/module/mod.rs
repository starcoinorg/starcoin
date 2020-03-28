// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_logger::prelude::*;

mod account_rpc;
mod node_rpc;
mod txpool_rpc;

pub use self::account_rpc::AccountRpcImpl;
pub use self::node_rpc::NodeRpcImpl;
pub use self::txpool_rpc::TxPoolRpcImpl;

pub fn map_err(err: anyhow::Error) -> jsonrpc_core::Error {
    //TODO error convert.
    error!("rpc return internal_error for: {:?}", err);
    jsonrpc_core::Error::internal_error()
}
