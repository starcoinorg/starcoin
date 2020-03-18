// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::future::{FutureExt, TryFutureExt};
use jsonrpc_core::BoxFuture;
use jsonrpc_core::Error;
use jsonrpc_derive::rpc;
use starcoin_rpc_api::{txpool::TxPoolApi, FutureResult};
use starcoin_types::transaction::SignedUserTransaction;
use traits::TxPoolAsyncService;

/// Re-export the API
pub use starcoin_rpc_api::txpool::*;

pub(crate) struct TxPoolRpcImpl<S>
where
    S: TxPoolAsyncService + 'static,
{
    service: S,
}

impl<S> TxPoolRpcImpl<S>
where
    S: TxPoolAsyncService,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> TxPoolApi for TxPoolRpcImpl<S>
where
    S: TxPoolAsyncService,
{
    fn submit_transaction(&self, txn: SignedUserTransaction) -> FutureResult<bool> {
        let fut = self.service.clone().add(txn).map_err(|_err| {
            //TODO
            Error::internal_error()
        });
        Box::new(fut.compat())
    }
}
