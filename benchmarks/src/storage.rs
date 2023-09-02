// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{BatchSize, Bencher};
use starcoin_crypto::HashValue;
use starcoin_schemadb::SchemaBatch;
use starcoin_storage::BlockTransactionInfoStore;
use starcoin_storage::Storage;
use starcoin_types::transaction::TransactionInfo;
use starcoin_vm_types::transaction::RichTransactionInfo;
use starcoin_vm_types::vm_status::KeptVMStatus;

/// Benchmarking support for storage.
pub struct StorageBencher {
    storage: Storage,
    num_accounts: usize,
    num_transactions: usize,
}

impl StorageBencher {
    /// The number of accounts created by default.
    pub const DEFAULT_NUM_ACCOUNTS: usize = 1000;

    /// The number of transactions created by default.
    pub const DEFAULT_NUM_TRANSACTIONS: usize = 2000;

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

    pub fn setup(&self) -> (Vec<(HashValue, Vec<HashValue>)>, Vec<RichTransactionInfo>) {
        let mut txn_infos = Vec::with_capacity(self.num_transactions);
        let mut txn_info_ids = Vec::with_capacity(self.num_transactions);
        for _i in 0..self.num_transactions {
            let transaction_info1 = TransactionInfo::new(
                HashValue::random(),
                HashValue::zero(),
                vec![].as_slice(),
                0,
                KeptVMStatus::Executed,
            );

            txn_info_ids.push((
                transaction_info1.transaction_hash(),
                vec![transaction_info1.id()],
            ));

            txn_infos.push(RichTransactionInfo::new(
                HashValue::zero(),
                rand::random(),
                transaction_info1,
                rand::random(),
                rand::random(),
            ));
        }
        (txn_info_ids, txn_infos)
    }

    /// Executes this state in a single block.
    fn execute(&self, data: (Vec<(HashValue, Vec<HashValue>)>, Vec<RichTransactionInfo>)) {
        let batch = SchemaBatch::new();
        self.storage
            .save_transaction_infos_batch(&data.1, &data.0, &batch)
            .unwrap();
        self.storage.ledger_db().write_schemas(batch).unwrap();
    }
    /// Runs the bencher.
    pub fn bench(&self, b: &mut Bencher) {
        b.iter_batched(
            || (self.setup(), self),
            |(data, bench)| bench.execute(data),
            BatchSize::LargeInput,
        )
    }
}
