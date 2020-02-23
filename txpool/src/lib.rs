// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate trace_time;
extern crate transaction_pool as tx_pool;
use crate::txpool::TxPoolImpl;
use actix::prelude::*;
use anyhow::{Error, Result};
use bus::{Broadcast, BusActor, Subscription};
use config::NodeConfig;
use std::sync::Arc;
use traits::TxPool;
use types::{system_events::SystemEvents, transaction, transaction::SignedUserTransaction};

mod pool;
mod tx_pool_service_impl;
mod txpool;

pub trait TxPoolService: Send + Sync {
    /// Import a set of transactions to the pool.
    ///
    /// Given blockchain and state access (Client)
    /// verifies and imports transactions to the pool.
    fn import_txns<C>(
        &self,
        client: C,
        txns: Vec<transaction::UnverifiedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>>
    where
        C: pool::NonceClient + pool::Client + Clone;

    /// Returns pending txns (if any)
    fn get_pending_txns<C>(
        &self,
        client: C,
        best_block_number: u64,
        best_block_timestamp: u64,
        max_len: u64,
    ) -> Option<Vec<Arc<pool::VerifiedTransaction>>>
    where
        C: pool::NonceClient;
}

pub struct TxPoolActor {
    pool: TxPoolImpl,
    bus: Addr<BusActor>,
}

impl TxPoolActor {
    pub fn launch(_node_config: &NodeConfig, bus: Addr<BusActor>) -> Result<TxPoolRef> {
        let actor_ref = Self {
            pool: TxPoolImpl::new(),
            bus,
        }
        .start();
        Ok(actor_ref.into())
    }
}

impl Actor for TxPoolActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<SystemEvents>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
    }
}

#[derive(Clone, Message)]
#[rtype(result = "Result<bool>")]
pub struct AddTransaction {
    pub txn: SignedUserTransaction,
}

#[derive(Clone, Message)]
#[rtype(result = "Result<Vec<SignedUserTransaction>>")]
pub struct GetPendingTransactions {}

impl Handler<AddTransaction> for TxPoolActor {
    type Result = Result<bool>;

    fn handle(&mut self, msg: AddTransaction, _ctx: &mut Self::Context) -> Self::Result {
        let new_tx = self.pool.add_tx(msg.txn.clone())?;
        if new_tx {
            self.bus.do_send(Broadcast {
                msg: SystemEvents::NewUserTransaction(msg.txn),
            });
        }
        return Ok(new_tx);
    }
}

impl Handler<GetPendingTransactions> for TxPoolActor {
    type Result = Result<Vec<SignedUserTransaction>>;

    fn handle(&mut self, _msg: GetPendingTransactions, _ctx: &mut Self::Context) -> Self::Result {
        self.pool.get_pending_txns()
    }
}

/// handle bus broadcast events.
impl Handler<SystemEvents> for TxPoolActor {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) {
        match msg {
            SystemEvents::NewHeadBlock(_block) => {
                // TODO remove block's txn from pool.
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct TxPoolRef(pub Addr<TxPoolActor>);

impl Into<Addr<TxPoolActor>> for TxPoolRef {
    fn into(self) -> Addr<TxPoolActor> {
        self.0
    }
}

impl Into<TxPoolRef> for Addr<TxPoolActor> {
    fn into(self) -> TxPoolRef {
        TxPoolRef(self)
    }
}

#[async_trait::async_trait]
impl TxPool for TxPoolRef {
    async fn add(self, txn: SignedUserTransaction) -> Result<bool> {
        self.0
            .send(AddTransaction { txn })
            .await
            .map_err(|e| Into::<Error>::into(e))?
    }

    async fn get_pending_txns(self) -> Result<Vec<SignedUserTransaction>> {
        self.0
            .send(GetPendingTransactions {})
            .await
            .map_err(|e| Into::<Error>::into(e))?
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
