// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecKVStore, CodecWriteBatch, ValueCodec};
use crate::TRANSACTION_PREFIX_NAME;
use crate::{define_storage, TransactionStore};
use anyhow::Result;
use bcs_ext::BCSCodec;
use crypto::HashValue;
use starcoin_types::transaction::Transaction;

define_storage!(
    TransactionStorage,
    HashValue,
    Transaction,
    TRANSACTION_PREFIX_NAME
);

impl ValueCodec for Transaction {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl TransactionStore for TransactionStorage {
    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>> {
        self.get(txn_hash)
    }

    fn save_transaction(&self, txn_info: Transaction) -> Result<()> {
        self.put(txn_info.id(), txn_info)
    }

    fn save_transaction_batch(&self, txn_vec: Vec<Transaction>) -> Result<()> {
        let batch =
            CodecWriteBatch::new_puts(txn_vec.into_iter().map(|txn| (txn.id(), txn)).collect());
        self.write_batch(batch)
    }
}

#[cfg(test)]
mod test;
