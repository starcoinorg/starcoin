// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::txpool::TxPool;
use actix::prelude::*;
use anyhow::Result;
use bus::{BusActor, Subscription};
use config::NodeConfig;
use network::{BroadcastTransactionMessage, NetworkActor};
use types::{system_events::SystemEvents, transaction::SignedUserTransaction};

mod txpool;

pub struct TxPoolActor {
    pool: TxPool,
    bus: Addr<BusActor>,
    network: Addr<NetworkActor>,
}

impl TxPoolActor {
    pub fn launch(
        _node_config: &NodeConfig,
        bus: Addr<BusActor>,
        network: Addr<NetworkActor>,
    ) -> Result<Addr<Self>> {
        let actor_ref = Self {
            pool: TxPool::new(),
            bus,
            network,
        }
        .start();
        Ok(actor_ref)
    }
}

impl Actor for TxPoolActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<SignedUserTransaction>();
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
            self.network
                .do_send(BroadcastTransactionMessage { tx: msg.tx });
        }
        return Ok(new_tx);
    }
}

/// handle bus broadcast Transaction.
impl Handler<SystemEvents> for TxPoolActor {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) {
        match msg {
            SystemEvents::NewUserTransaction(txn) => match self.pool.add_tx(txn) {
                Ok(_) => {}
                Err(err) => println!("Add tx to pool error:{:?}", err),
            },
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
