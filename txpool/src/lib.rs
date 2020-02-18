// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate transaction_pool as tx_pool;
use crate::txpool::TxPool;
use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor, Subscription};
use config::NodeConfig;
use types::{system_events::SystemEvents, transaction::SignedUserTransaction};

mod pool;
mod txpool;
pub struct TxPoolActor {
    pool: TxPool,
    bus: Addr<BusActor>,
}

impl TxPoolActor {
    pub fn launch(_node_config: &NodeConfig, bus: Addr<BusActor>) -> Result<Addr<Self>> {
        let actor_ref = Self {
            pool: TxPool::new(),
            bus,
        }
        .start();
        Ok(actor_ref)
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
pub struct SubmitTransactionMessage {
    pub tx: SignedUserTransaction,
}

impl Handler<SubmitTransactionMessage> for TxPoolActor {
    type Result = Result<bool>;

    fn handle(&mut self, msg: SubmitTransactionMessage, _ctx: &mut Self::Context) -> Self::Result {
        let new_tx = self.pool.add_tx(msg.tx.clone())?;
        if new_tx {
            self.bus.do_send(Broadcast {
                msg: SystemEvents::NewUserTransaction(msg.tx),
            });
        }
        return Ok(new_tx);
    }
}

/// handle bus broadcast Transaction.
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
