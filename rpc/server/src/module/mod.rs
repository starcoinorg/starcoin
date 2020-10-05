// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_logger::prelude::*;

mod account_rpc;
mod chain_rpc;
mod debug_rpc;
mod dev_rpc;
mod miner_rpc;
mod node_manager_rpc;
mod node_rpc;
mod pubsub;
mod state_rpc;
mod txpool_rpc;

pub use self::account_rpc::AccountRpcImpl;
pub use self::chain_rpc::ChainRpcImpl;
pub use self::debug_rpc::DebugRpcImpl;
pub use self::dev_rpc::DevRpcImpl;
pub use self::miner_rpc::MinerRpcImpl;
pub use self::node_manager_rpc::NodeManagerRpcImpl;
pub use self::node_rpc::NodeRpcImpl;
pub use self::pubsub::{PubSubImpl, PubSubService};
pub use self::state_rpc::StateRpcImpl;
pub use self::txpool_rpc::TxPoolRpcImpl;
use starcoin_account_api::error::AccountError;

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

impl From<AccountError> for RpcError {
    fn from(err: AccountError) -> Self {
        match err {
            AccountError::StoreError(_) => RpcError::InternalError,
            e => RpcError::InvalidRequest(format!("{:?}", e)),
        }
    }
}
