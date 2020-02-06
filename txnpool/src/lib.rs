// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::txnpool::TxnPool;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use types::transaction::SignedTransaction;

mod txnpool;

pub struct TxnPoolActor {
    pool: TxnPool,
}

impl TxnPoolActor {
    pub fn launch(_node_config: &NodeConfig) -> Result<Addr<TxnPoolActor>> {
        Ok(TxnPoolActor {
            pool: TxnPool::new(),
        }
        .start())
    }
}

impl Actor for TxnPoolActor {
    type Context = Context<Self>;
}

#[derive(PartialEq)]
pub enum TxnPoolStatusCode {
    Valid,
    TxnPoolFull,
}

#[derive(MessageResponse, PartialEq)]
pub struct TxnPoolStatus {
    code: TxnPoolStatusCode,
}

#[derive(Message)]
#[rtype(result = "TxnPoolStatus")]
pub struct SubmitTransactionMessage {
    txn: SignedTransaction,
}

impl Handler<SubmitTransactionMessage> for TxnPoolActor {
    type Result = TxnPoolStatus;

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
