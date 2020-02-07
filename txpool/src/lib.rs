// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::txpool::TxPool;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use network::NetworkActor;
use types::transaction::SignedTransaction;

mod txpool;

pub struct TxPoolActor {
    pool: TxPool,
}

impl TxPoolActor {
    pub fn launch(
        _node_config: &NodeConfig,
        network: Addr<NetworkActor>,
    ) -> Result<Addr<TxPoolActor>> {
        Ok(TxPoolActor {
            pool: TxPool::new(network),
        }
        .start())
    }
}

impl Actor for TxPoolActor {
    type Context = Context<Self>;
}

#[derive(PartialEq)]
pub enum TxPoolStatusCode {
    Valid,
    TxPoolFull,
}

#[derive(MessageResponse, PartialEq)]
pub struct TxPoolStatus {
    code: TxPoolStatusCode,
}

#[derive(Message)]
#[rtype(result = "TxPoolStatus")]
pub struct SubmitTransactionMessage {
    txn: SignedTransaction,
}

impl Handler<SubmitTransactionMessage> for TxPoolActor {
    type Result = TxPoolStatus;

    fn handle(&mut self, msg: SubmitTransactionMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.pool.add_transaction(msg.txn)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
