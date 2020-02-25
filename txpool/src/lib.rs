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
use storage::StarcoinStorage;
use traits::TxPoolAsyncService;
use types::{system_events::SystemEvents, transaction::SignedUserTransaction};

mod pool;
mod tx_pool_service_impl;
mod txpool;

pub use tx_pool_service_impl::{CachedSeqNumberClient, TxPool, TxPoolRef};

#[cfg(test)]
mod test;

/// TODO: deprecate
pub struct TxPoolActor {
    pool: TxPoolImpl,
    bus: Addr<BusActor>,
}

impl TxPoolActor {
    // pub fn launch(
    //     _node_config: Arc<NodeConfig>,
    //     bus: Addr<BusActor>,
    //     _storage: Arc<StarcoinStorage>,
    // ) -> Result<TxPoolRef> {
    //     let actor_ref = Self {
    //         pool: TxPoolImpl::new(),
    //         bus,
    //     }
    //     .start();
    //     Ok(actor_ref.into())
    // }
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
