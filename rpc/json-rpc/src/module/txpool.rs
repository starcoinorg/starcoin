// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use futures::future::{FutureExt, TryFutureExt};
use jsonrpc_core::BoxFuture;
use jsonrpc_core::Error;
use jsonrpc_derive::rpc;
use txpool::{SubmitTransactionMessage, TxPoolActor};
use types::transaction::SignedUserTransaction;

#[rpc(server)]
pub trait TxPoolRpc {
    #[rpc(name = "submit_transaction")]
    fn submit_transaction(&self, tx: SignedUserTransaction) -> BoxFuture<bool>;
}

pub(crate) struct TxPoolRpcImpl {
    actor_ref: Addr<TxPoolActor>,
}

impl TxPoolRpcImpl {
    pub fn new(actor_ref: Addr<TxPoolActor>) -> Self {
        Self { actor_ref }
    }
}

impl TxPoolRpc for TxPoolRpcImpl {
    fn submit_transaction(&self, tx: SignedUserTransaction) -> BoxFuture<bool> {
        let fut = self
            .actor_ref
            .send(SubmitTransactionMessage { tx })
            .map(|res|
                //TODO
                res.unwrap())
            .map_err(|_err| {
                //TODO
                Error::internal_error()
            });
        Box::new(fut.boxed().compat())
    }
}
