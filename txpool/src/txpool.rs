// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use types::transaction::SignedUserTransaction;

pub struct TxPool {
    // TODO
    txs: Vec<SignedUserTransaction>,
}

impl TxPool {
    pub fn new() -> Self {
        Self { txs: vec![] }
    }

    /// Add tx to pool and return it is a new transaction.
    pub fn add_tx(&mut self, txn: SignedUserTransaction) -> Result<bool> {
        // TODO verify txn.
        // TODO check transaction is exist, only broadcast no exist transaction.
        self.txs.push(txn.clone());
        return Ok(true);
    }
}
