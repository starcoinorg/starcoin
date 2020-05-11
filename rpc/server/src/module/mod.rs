// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_logger::prelude::*;

mod chain_rpc;
mod debug_rpc;
mod node_rpc;
mod pubsub;
mod state_rpc;
mod txpool_rpc;
mod wallet_rpc;

pub use self::chain_rpc::ChainRpcImpl;
pub use self::debug_rpc::DebugRpcImpl;
pub use self::node_rpc::NodeRpcImpl;
pub use self::pubsub::{PubSubImpl, PubSubService};
pub use self::state_rpc::StateRpcImpl;
pub use self::txpool_rpc::TxPoolRpcImpl;
pub use self::wallet_rpc::WalletRpcImpl;

use starcoin_wallet_api::error::AccountServiceError;

pub fn map_err(err: anyhow::Error) -> jsonrpc_core::Error {
    //TODO error convert.
    error!("rpc return internal_error for: {:?}", err);
    jsonrpc_core::Error::internal_error()
}

pub fn to_invalid_param_err<E>(err: E) -> jsonrpc_core::Error
where
    E: Into<anyhow::Error>,
{
    let anyhow_err: anyhow::Error = err.into();
    let message = format!("Invalid param error: {:?}", anyhow_err);
    jsonrpc_core::Error::invalid_params(message)
}

pub fn map_rpc_err(err: RpcError) -> jsonrpc_core::Error {
    match err {
        RpcError::InternalError => jsonrpc_core::Error::internal_error(),
        RpcError::InvalidRequest(message) => jsonrpc_core::Error::invalid_params(message),
    }
}

#[derive(Debug)]
pub enum RpcError {
    InternalError,
    InvalidRequest(String),
}

impl From<AccountServiceError> for RpcError {
    fn from(err: AccountServiceError) -> Self {
        match err {
            AccountServiceError::AccountError(_) => RpcError::InternalError,
            AccountServiceError::OtherError(_) => RpcError::InternalError,
            e @ _ => RpcError::InvalidRequest(format!("{}", e)),
        }
    }
}
