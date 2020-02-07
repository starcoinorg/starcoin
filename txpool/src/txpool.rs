// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{TxPoolStatus, TxPoolStatusCode};
use actix::prelude::*;
use network::{BroadcastTransactionMessage, NetworkActor};
use types::transaction::SignedTransaction;

pub struct TxPool {
    //TODO
    transactions: Vec<SignedTransaction>,
    network: Addr<NetworkActor>,
}

impl TxPool {
    pub fn new(network: Addr<NetworkActor>) -> Self {
        Self {
            transactions: vec![],
            network,
        }
    }
    pub fn add_transaction(&mut self, transaction: SignedTransaction) -> TxPoolStatus {
        //TODO check transaction is exist, only broadcast no exist transaction.
        self.transactions.push(transaction.clone());
        self.broadcast_transaction(transaction);
        TxPoolStatus {
            code: TxPoolStatusCode::Valid,
        }
    }

    fn broadcast_transaction(&self, transaction: SignedTransaction) {
        self.network
            .do_send(BroadcastTransactionMessage { transaction })
    }
}
