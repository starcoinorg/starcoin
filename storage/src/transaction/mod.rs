// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::storage::{CodecStorage, ValueCodec};
use crate::TRANSACTION_PREFIX_NAME;
use crate::{define_storage, TransactionStore};
use anyhow::Error;
use anyhow::Result;
use crypto::HashValue;
use scs::SCSCodec;
use starcoin_types::transaction::Transaction;
use std::sync::Arc;

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
    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>, Error> {
        self.store.get(txn_hash)
    }

    fn save_transaction(&self, txn_info: Transaction) -> Result<(), Error> {
        self.store.put(txn_info.id(), txn_info)
    }

    fn save_transaction_batch(&self, txn_vec: Vec<Transaction>) -> Result<(), Error> {
        let mut batch = WriteBatch::new_with_name(TRANSACTION_PREFIX_NAME);
        for transaction in txn_vec {
            batch.put(transaction.id(), transaction)?;
        }
        self.store.write_batch(batch)
    }
}
