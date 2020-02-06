// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{TxnPoolStatus, TxnPoolStatusCode};
use types::transaction::SignedTransaction;

pub struct TxnPool {
    //TODO
    txns: Vec<SignedTransaction>,
}

impl TxnPool {
    pub fn new() -> Self {
        Self { txns: vec![] }
    }
    pub fn add_transaction(&mut self, txn: SignedTransaction) -> TxnPoolStatus {
        self.txns.push(txn);
        TxnPoolStatus {
            code: TxnPoolStatusCode::Valid,
        }
    }
}
