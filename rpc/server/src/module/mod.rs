// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod account_rpc;
mod chain_rpc;
mod contract_rpc;
mod debug_rpc;
mod helpers;
mod miner_rpc;
mod network_manager_rpc;
mod node_manager_rpc;
mod node_rpc;
mod pubsub;
mod state_rpc;
mod sync_manager_rpc;
mod txfactory_rpc;
mod txpool_rpc;

pub use self::account_rpc::AccountRpcImpl;
pub use self::chain_rpc::ChainRpcImpl;
pub use self::contract_rpc::ContractRpcImpl;
pub use self::debug_rpc::DebugRpcImpl;
pub use self::miner_rpc::MinerRpcImpl;
pub use self::network_manager_rpc::NetworkManagerRpcImpl;
pub use self::node_manager_rpc::NodeManagerRpcImpl;
pub use self::node_rpc::NodeRpcImpl;
pub use self::pubsub::{PubSubImpl, PubSubService, PubSubServiceFactory};
pub use self::state_rpc::StateRpcImpl;
pub use self::sync_manager_rpc::SyncManagerRpcImpl;
pub use self::txfactory_rpc::TxFactoryStatusHandle;
pub use self::txpool_rpc::TxPoolRpcImpl;

use actix::MailboxError;
use anyhow::Error;
use hex::FromHexError;
use jsonrpc_core::ErrorCode;
use starcoin_account_api::error::AccountError;
use starcoin_rpc_api::types::TransactionStatusView;
use starcoin_vm_types::transaction::{CallError, TransactionError, TransactionStatus};
use starcoin_vm_types::vm_status::VMStatus;

pub fn map_err(err: anyhow::Error) -> jsonrpc_core::Error {
    // if err is a jsonrpc error, return directly.
    if err.is::<jsonrpc_core::Error>() {
        return err.downcast::<jsonrpc_core::Error>().unwrap();
    }
    // TODO: add more error downcasting here
    let rpc_error: RpcError = if err.is::<TransactionError>() {
        err.downcast::<TransactionError>().unwrap().into()
    } else if err.is::<bcs_ext::Error>() {
        err.downcast::<bcs_ext::Error>().unwrap().into()
    } else if err.is::<AccountError>() {
        err.downcast::<AccountError>().unwrap().into()
    } else if err.is::<MailboxError>() {
        err.downcast::<MailboxError>().unwrap().into()
    } else if err.is::<VMStatus>() {
        err.downcast::<VMStatus>().unwrap().into()
    } else {
        err.into()
    };
    rpc_error.into()
}

fn convert_to_rpc_error<T: Into<RpcError>>(err: T) -> jsonrpc_core::Error {
    let err = err.into();
    err.into()
}

/// A wrapper for jsonrpc error.
/// It's necessary because
/// only traits defined in the current crate can be implemented for arbitrary types.
#[derive(Debug)]
struct RpcError(jsonrpc_core::Error);

#[allow(clippy::from_over_into)]
impl Into<jsonrpc_core::Error> for RpcError {
    fn into(self) -> jsonrpc_core::Error {
        self.0
    }
}

impl From<anyhow::Error> for RpcError {
    fn from(e: Error) -> Self {
        RpcError(jsonrpc_core::Error {
            code: jsonrpc_core::ErrorCode::InternalError,
            message: e.to_string(),
            data: None,
        })
    }
}

const TXN_ERROR_BASE: i64 = -50000;
const ACCOUNT_ERROR_BASE: i64 = -60000;

impl From<AccountError> for RpcError {
    fn from(err: AccountError) -> Self {
        let rpc_error = match err {
            AccountError::StoreError(error) => jsonrpc_core::Error {
                code: ErrorCode::ServerError(ACCOUNT_ERROR_BASE),
                message: error.to_string(),
                data: None,
            },
            e => jsonrpc_core::Error {
                code: ErrorCode::InvalidParams,
                message: e.to_string(),
                data: None,
            },
        };
        RpcError(rpc_error)
    }
}

impl From<TransactionError> for RpcError {
    fn from(err: TransactionError) -> Self {
        let err_message = err.to_string();
        let (err_code, err_data) = match err {
            TransactionError::AlreadyImported
            | TransactionError::Old
            | TransactionError::InsufficientGasPrice { .. }
            | TransactionError::TooCheapToReplace { .. }
            | TransactionError::InsufficientGas { .. }
            | TransactionError::InsufficientBalance { .. }
            | TransactionError::GasLimitExceeded { .. }
            | TransactionError::SenderBanned
            | TransactionError::RecipientBanned
            | TransactionError::CodeBanned
            | TransactionError::InvalidChainId
            | TransactionError::InvalidSignature(..)
            | TransactionError::NotAllowed
            | TransactionError::TooBig => (ErrorCode::InvalidParams, None),
            TransactionError::LimitReached => (ErrorCode::ServerError(TXN_ERROR_BASE), None),
            TransactionError::CallErr(call_err) => match call_err {
                CallError::TransactionNotFound => (ErrorCode::InvalidParams, None),
                CallError::StatePruned | CallError::StateCorrupt => {
                    (ErrorCode::ServerError(TXN_ERROR_BASE + 1), None)
                }
                CallError::ExecutionError(vm_status) => (
                    ErrorCode::ServerError(TXN_ERROR_BASE + 2),
                    Some(
                        // translate to jsonrpc types
                        serde_json::to_value(TransactionStatusView::from(TransactionStatus::from(
                            vm_status,
                        )))
                        .expect("vm status to json should be ok"),
                    ),
                ),
            },
        };
        RpcError(jsonrpc_core::Error {
            code: err_code,
            message: err_message,
            data: err_data,
        })
    }
}

impl From<hex::FromHexError> for RpcError {
    fn from(err: FromHexError) -> Self {
        RpcError(jsonrpc_core::Error {
            code: ErrorCode::InvalidParams,
            message: err.to_string(),
            data: None,
        })
    }
}
impl From<bcs_ext::Error> for RpcError {
    fn from(err: bcs_ext::Error) -> Self {
        RpcError(jsonrpc_core::Error {
            code: ErrorCode::InvalidParams,
            message: err.to_string(),
            data: None,
        })
    }
}

impl From<MailboxError> for RpcError {
    fn from(err: MailboxError) -> Self {
        RpcError(jsonrpc_core::Error {
            code: ErrorCode::InternalError,
            message: err.to_string(),
            data: None,
        })
    }
}

impl From<VMStatus> for RpcError {
    fn from(vm_status: VMStatus) -> Self {
        RpcError(jsonrpc_core::Error {
            code: ErrorCode::InvalidParams,
            message: vm_status.to_string(),
            data: Some(
                // use jsonrpc types do serialization.
                serde_json::to_value(TransactionStatusView::from(TransactionStatus::from(
                    vm_status,
                )))
                .expect("vm status to json should be ok"),
            ),
        })
    }
}

pub fn to_invalid_param_err<E>(err: E) -> jsonrpc_core::Error
where
    E: Into<anyhow::Error>,
{
    let anyhow_err: anyhow::Error = err.into();
    let message = format!("Invalid param error: {:?}", anyhow_err);
    jsonrpc_core::Error::invalid_params(message)
}
