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

use actix::MailboxError;
use hex::FromHexError;
use jsonrpsee::types::{
    error::{INTERNAL_ERROR_CODE, INVALID_PARAMS_CODE, INVALID_REQUEST_CODE},
    ErrorObjectOwned,
};
use serde_json::Value;
use starcoin_account_api::error::AccountError;
use starcoin_rpc_api::types::TransactionStatusView;
use starcoin_types::multi_transaction::MultiTransactionError;
use starcoin_vm2_types::view::TransactionStatusView as TransactionStatusView2;
use starcoin_vm2_vm_types::{
    transaction::{
        CallError as CallError2, TransactionError as TransactionError2,
        TransactionStatus as TransactionStatus2,
    },
    vm_status::VMStatus as VMStatus2,
};
use starcoin_vm_types::{
    transaction::{CallError, TransactionError, TransactionStatus},
    vm_status::VMStatus,
};

pub use self::account_rpc::AccountRpcImpl;
pub use self::chain_rpc::ChainRpcImpl;
pub use self::contract_rpc::ContractRpcImpl;
pub use self::debug_rpc::DebugRpcImpl;
pub use self::miner_rpc::MinerRpcImpl;
pub use self::network_manager_rpc::NetworkManagerRpcImpl;
pub use self::node_manager_rpc::NodeManagerRpcImpl;
pub use self::node_rpc::NodeRpcImpl;
pub use self::pubsub::{pubsub_methods, PubSubImpl, PubSubService, PubSubServiceFactory};
pub use self::state_rpc::StateRpcImpl;
pub use self::sync_manager_rpc::SyncManagerRpcImpl;
pub use self::txfactory_rpc::TxFactoryStatusHandle;
pub use self::txpool_rpc::TxPoolRpcImpl;

pub fn map_err(err: anyhow::Error) -> anyhow::Error {
    err
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
struct RpcErrorWithCode {
    code: i32,
    message: String,
}

const TXN_ERROR_BASE: i32 = -50000;
const ACCOUNT_ERROR_BASE: i32 = -60000;
const INTERNAL_ERROR_MESSAGE: &str = "Internal error";

fn to_rpc_error_with_code(code: i32, message: impl Into<String>) -> anyhow::Error {
    RpcErrorWithCode {
        code,
        message: message.into(),
    }
    .into()
}

fn owned_error(code: i32, message: impl Into<String>, data: Option<Value>) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(code, message.into(), data)
}

fn internal_error_response(error: impl std::fmt::Debug) -> ErrorObjectOwned {
    log::error!("internal rpc error: {:?}", error);
    owned_error(INTERNAL_ERROR_CODE, INTERNAL_ERROR_MESSAGE, None)
}

fn invalid_params_error(message: impl Into<String>) -> ErrorObjectOwned {
    owned_error(INVALID_PARAMS_CODE, message, None)
}

fn execution_status_data(vm_status: VMStatus) -> Option<Value> {
    Some(
        serde_json::to_value(TransactionStatusView::from(TransactionStatus::from(
            vm_status,
        )))
        .expect("vm status should serialize"),
    )
}

fn execution_status_data2(vm_status: VMStatus2) -> Option<Value> {
    Some(
        serde_json::to_value(TransactionStatusView2::from(TransactionStatus2::from(
            vm_status,
        )))
        .expect("vm2 status should serialize"),
    )
}

fn map_account_error(err: AccountError) -> ErrorObjectOwned {
    match err {
        AccountError::StoreError(error) => {
            log::error!("account store error: {:?}", error);
            owned_error(ACCOUNT_ERROR_BASE, "Account store error", None)
        }
        other => invalid_params_error(other.to_string()),
    }
}

fn map_transaction_call_error(message: String, err: CallError) -> ErrorObjectOwned {
    match err {
        CallError::TransactionNotFound => invalid_params_error(message),
        CallError::StatePruned | CallError::StateCorrupt => {
            owned_error(TXN_ERROR_BASE + 1, message, None)
        }
        CallError::ExecutionError(vm_status) => owned_error(
            TXN_ERROR_BASE + 2,
            message,
            execution_status_data(vm_status),
        ),
    }
}

fn map_transaction_error(err: TransactionError) -> ErrorObjectOwned {
    let message = err.to_string();
    match err {
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
        | TransactionError::InvalidSignature(_)
        | TransactionError::NotAllowed
        | TransactionError::TooBig => invalid_params_error(message),
        TransactionError::LimitReached(_) => owned_error(TXN_ERROR_BASE, message, None),
        TransactionError::CallErr(call_err) => map_transaction_call_error(message, call_err),
        TransactionError::APIInterrupted(reason) => {
            internal_error_response(format!("transaction api interrupted: {reason}"))
        }
    }
}

fn map_transaction_call_error2(message: String, err: CallError2) -> ErrorObjectOwned {
    match err {
        CallError2::TransactionNotFound => invalid_params_error(message),
        CallError2::StatePruned | CallError2::StateCorrupt => {
            owned_error(TXN_ERROR_BASE + 1, message, None)
        }
        CallError2::ExecutionError(vm_status) => owned_error(
            TXN_ERROR_BASE + 2,
            message,
            execution_status_data2(vm_status),
        ),
    }
}

fn map_transaction_error2(err: TransactionError2) -> ErrorObjectOwned {
    let message = err.to_string();
    match err {
        TransactionError2::AlreadyImported
        | TransactionError2::Old
        | TransactionError2::InsufficientGasPrice { .. }
        | TransactionError2::TooCheapToReplace { .. }
        | TransactionError2::InsufficientGas { .. }
        | TransactionError2::InsufficientBalance { .. }
        | TransactionError2::GasLimitExceeded { .. }
        | TransactionError2::SenderBanned
        | TransactionError2::RecipientBanned
        | TransactionError2::CodeBanned
        | TransactionError2::InvalidChainId
        | TransactionError2::InvalidSignature(_)
        | TransactionError2::NotAllowed
        | TransactionError2::TooBig => invalid_params_error(message),
        TransactionError2::LimitReached => owned_error(TXN_ERROR_BASE, message, None),
        TransactionError2::CallErr(call_err) => map_transaction_call_error2(message, call_err),
        TransactionError2::APIInterrupted(reason) => {
            internal_error_response(format!("vm2 transaction api interrupted: {reason}"))
        }
    }
}

fn map_multi_transaction_error(err: MultiTransactionError) -> ErrorObjectOwned {
    match err {
        MultiTransactionError::VM1(err) => map_transaction_error(err),
        MultiTransactionError::VM2(err) => map_transaction_error2(err),
        MultiTransactionError::APIInterrupted(reason) => {
            internal_error_response(format!("multi transaction api interrupted: {reason}"))
        }
    }
}

pub fn map_jsonrpc_err(err: anyhow::Error) -> ErrorObjectOwned {
    let err = match err.downcast::<RpcErrorWithCode>() {
        Ok(err) => return owned_error(err.code, err.message, None),
        Err(err) => err,
    };
    let err = match err.downcast::<ErrorObjectOwned>() {
        Ok(err) => return err,
        Err(err) => err,
    };
    let err = match err.downcast::<TransactionError>() {
        Ok(err) => return map_transaction_error(err),
        Err(err) => err,
    };
    let err = match err.downcast::<TransactionError2>() {
        Ok(err) => return map_transaction_error2(err),
        Err(err) => err,
    };
    let err = match err.downcast::<MultiTransactionError>() {
        Ok(err) => return map_multi_transaction_error(err),
        Err(err) => err,
    };
    let err = match err.downcast::<FromHexError>() {
        Ok(err) => return invalid_params_error(err.to_string()),
        Err(err) => err,
    };
    let err = match err.downcast::<bcs_ext::Error>() {
        Ok(err) => return invalid_params_error(err.to_string()),
        Err(err) => err,
    };
    let err = match err.downcast::<AccountError>() {
        Ok(err) => return map_account_error(err),
        Err(err) => err,
    };
    let err = match err.downcast::<MailboxError>() {
        Ok(err) => return internal_error_response(err),
        Err(err) => err,
    };
    let err = match err.downcast::<VMStatus>() {
        Ok(err) => {
            return owned_error(
                INVALID_PARAMS_CODE,
                err.to_string(),
                execution_status_data(err),
            );
        }
        Err(err) => err,
    };
    let err = match err.downcast::<VMStatus2>() {
        Ok(err) => {
            return owned_error(
                INVALID_PARAMS_CODE,
                err.to_string(),
                execution_status_data2(err),
            );
        }
        Err(err) => err,
    };

    internal_error_response(err)
}

pub fn convert_to_rpc_error<T: Into<anyhow::Error>>(err: T) -> anyhow::Error {
    err.into()
}

pub fn to_invalid_param_err<E>(err: E) -> anyhow::Error
where
    E: Into<anyhow::Error>,
{
    to_rpc_error_with_code(INVALID_PARAMS_CODE, err.into().to_string())
}

pub fn to_invalid_request_err<E>(err: E) -> anyhow::Error
where
    E: Into<anyhow::Error>,
{
    to_rpc_error_with_code(INVALID_REQUEST_CODE, err.into().to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        map_jsonrpc_err, to_invalid_param_err, to_invalid_request_err, ACCOUNT_ERROR_BASE,
        INTERNAL_ERROR_MESSAGE, TXN_ERROR_BASE,
    };
    use jsonrpsee::types::error::{INTERNAL_ERROR_CODE, INVALID_PARAMS_CODE, INVALID_REQUEST_CODE};
    use starcoin_account_api::error::AccountError;
    use starcoin_vm_types::{
        transaction::TransactionError,
        vm_status::{StatusCode, VMStatus},
    };

    #[test]
    fn preserves_invalid_param_code() {
        let err = map_jsonrpc_err(to_invalid_param_err(anyhow::anyhow!("bad input")));
        assert_eq!(err.code(), INVALID_PARAMS_CODE);
        assert_eq!(err.message(), "bad input");
    }

    #[test]
    fn preserves_invalid_request_code() {
        let err = map_jsonrpc_err(to_invalid_request_err(anyhow::anyhow!("invalid request")));
        assert_eq!(err.code(), INVALID_REQUEST_CODE);
        assert_eq!(err.message(), "invalid request");
    }

    #[test]
    fn defaults_other_errors_to_internal_error() {
        let err = map_jsonrpc_err(anyhow::anyhow!("boom"));
        assert_eq!(err.code(), INTERNAL_ERROR_CODE);
        assert_eq!(err.message(), INTERNAL_ERROR_MESSAGE);
    }

    #[test]
    fn maps_transaction_errors_to_domain_codes() {
        let err = map_jsonrpc_err(TransactionError::LimitReached("txpool full".into()).into());
        assert_eq!(err.code(), TXN_ERROR_BASE);
        assert!(err.message().contains("txpool full"));
    }

    #[test]
    fn preserves_transaction_execution_error_payloads() {
        let err = map_jsonrpc_err(
            TransactionError::CallErr(starcoin_vm_types::transaction::CallError::ExecutionError(
                VMStatus::Error(StatusCode::INVALID_SIGNATURE),
            ))
            .into(),
        );
        assert_eq!(err.code(), TXN_ERROR_BASE + 2);
        assert!(err.data().is_some());
    }

    #[test]
    fn sanitizes_account_store_errors() {
        let err = map_jsonrpc_err(AccountError::StoreError(anyhow::anyhow!("db down")).into());
        assert_eq!(err.code(), ACCOUNT_ERROR_BASE);
        assert_eq!(err.message(), "Account store error");
    }
}
