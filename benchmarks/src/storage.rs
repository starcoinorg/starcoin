// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{BatchSize, Bencher};
use crypto::HashValue;
use starcoin_storage::BlockTransactionInfoStore;
use starcoin_storage::Storage;
use starcoin_vm_types::transaction::RichTransactionInfo;
use starcoin_vm_types::vm_status::KeptVMStatus;
use types::transaction::TransactionInfo;

/// Benchmarking support for storage.
pub struct StorageBencher {
    storage: Storage,
    num_accounts: usize,
    num_transactions: usize,
}

impl StorageBencher {
    /// The number of accounts created by default.
    pub const DEFAULT_NUM_ACCOUNTS: usize = 10;

    /// The number of transactions created by default.
    pub const DEFAULT_NUM_TRANSACTIONS: usize = 20;

    /// Creates a new transaction bencher with default settings.
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            num_accounts: Self::DEFAULT_NUM_ACCOUNTS,
            num_transactions: Self::DEFAULT_NUM_TRANSACTIONS,
        }
    }

    /// Sets a custom number of accounts.
    pub fn num_accounts(&mut self, num_accounts: usize) -> &mut Self {
        self.num_accounts = num_accounts;
        self
    }

    /// Sets a custom number of transactions.
    pub fn num_transactions(&mut self, num_transactions: usize) -> &mut Self {
        self.num_transactions = num_transactions;
        self
    }
    /// Executes this state in a single block.
    fn execute(&self) {
        for _i in 0..self.num_transactions {
            let transaction_info1 = TransactionInfo::new(
                HashValue::random(),
                HashValue::zero(),
                vec![].as_slice(),
                0,
                KeptVMStatus::Executed,
            );

            self.storage
                .save_transaction_infos(vec![RichTransactionInfo::new(
                    HashValue::zero(),
                    rand::random(),
                    transaction_info1,
                    rand::random(),
                    rand::random(),
                )])
                .unwrap();
        }
    }
    /// Runs the bencher.
    pub fn bench(&self, b: &mut Bencher) {
        b.iter_batched(|| self, |bench| bench.execute(), BatchSize::LargeInput)
    }
}
