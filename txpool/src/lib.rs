// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::txpool::TxPool;
use actix::prelude::*;
use anyhow::Result;
use bus::{BusActor, Subscription};
use config::NodeConfig;
use network::{BroadcastTransactionMessage, NetworkActor};
use types::transaction::SignedTransaction;

mod txpool;

pub struct TxPoolActor {
    pool: TxPool,
    network: Addr<NetworkActor>,
}

impl TxPoolActor {
    pub async fn launch(
        _node_config: &NodeConfig,
        bus: Addr<BusActor>,
        network: Addr<NetworkActor>,
    ) -> Result<Addr<TxPoolActor>> {
        let addr = TxPoolActor {
            pool: TxPool::new(),
            network,
        }
        .start();
        let recipient = addr.clone().recipient::<SignedTransaction>();
        bus.send(Subscription { recipient }).await?;
        Ok(addr)
    }
}

impl Actor for TxPoolActor {
    type Context = Context<Self>;
}

#[derive(Clone, Message)]
#[rtype(result = "Result<bool>")]
pub struct SubmitTransactionMessage {
    tx: SignedTransaction,
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
impl Handler<SignedTransaction> for TxPoolActor {
    type Result = ();

    fn handle(&mut self, msg: SignedTransaction, _ctx: &mut Self::Context) {
        match self.pool.add_tx(msg) {
            Ok(_) => {}
            Err(err) => println!("Add tx to pool error:{:?}", err),
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
