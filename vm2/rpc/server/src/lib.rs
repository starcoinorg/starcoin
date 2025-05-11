// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Error;

pub(crate) fn map_err(err: anyhow::Error) -> jsonrpc_core::Error {
    // if err is a jsonrpc error, return directly.
    if err.is::<jsonrpc_core::Error>() {
        return err.downcast::<jsonrpc_core::Error>().unwrap();
    }
    // TODO: add more error downcasting here

    let rpc_error: RpcError =  /* if err.is::<TransactionError>() {
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
    }; */
        err.into();

    rpc_error.into()
}

#[allow(dead_code)]
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
        Self(jsonrpc_core::Error {
            code: jsonrpc_core::ErrorCode::InternalError,
            message: e.to_string(),
            data: None,
        })
    }
}

mod account_rpc;
mod helpers;
mod state_rpc;
