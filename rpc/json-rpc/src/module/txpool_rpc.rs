// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::future::{FutureExt, TryFutureExt};
use jsonrpc_core::BoxFuture;
use jsonrpc_core::Error;
use jsonrpc_derive::rpc;
use traits::TxPoolAsyncService;
use types::transaction::SignedUserTransaction;

#[rpc(server)]
pub trait TxPoolRpc {
    #[rpc(name = "submit_transaction")]
    fn submit_transaction(&self, tx: SignedUserTransaction) -> BoxFuture<bool>;
}

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

impl<S> TxPoolRpc for TxPoolRpcImpl<S>
where
    S: TxPoolAsyncService,
{
    fn submit_transaction(&self, txn: SignedUserTransaction) -> BoxFuture<bool> {
        let fut = self.service.clone().add(txn).map_err(|_err| {
            //TODO
            Error::internal_error()
        });
        Box::new(fut.boxed().compat())
    }
}
