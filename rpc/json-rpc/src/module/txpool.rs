// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::future::{FutureExt, TryFutureExt};
use jsonrpc_core::BoxFuture;
use jsonrpc_core::Error;
use jsonrpc_derive::rpc;
use traits::TxPoolAsyncService;
use txpool::TxPoolRef;
use types::transaction::SignedUserTransaction;

#[rpc(server)]
pub trait TxPoolRpc {
    #[rpc(name = "submit_transaction")]
    fn submit_transaction(&self, tx: SignedUserTransaction) -> BoxFuture<bool>;
}

pub(crate) struct TxPoolRpcImpl {
    actor_ref: TxPoolRef,
}

impl TxPoolRpcImpl {
    pub fn new(actor_ref: TxPoolRef) -> Self {
        Self { actor_ref }
    }
}

impl TxPoolRpc for TxPoolRpcImpl {
    fn submit_transaction(&self, txn: SignedUserTransaction) -> BoxFuture<bool> {
        let fut = self.actor_ref.clone().add(txn).map_err(|_err| {
            //TODO
            Error::internal_error()
        });
        Box::new(fut.boxed().compat())
    }
}
