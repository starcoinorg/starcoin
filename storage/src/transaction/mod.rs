// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod legacy;

use crate::storage::{CodecKVStore, CodecWriteBatch, ValueCodec};
use crate::{define_storage, TransactionStore, TRANSACTION_PREFIX_NAME_V2};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::transaction::StcTransaction;

define_storage!(
    StcTransactionStorage,
    HashValue,
    StcTransaction,
    TRANSACTION_PREFIX_NAME_V2
);

impl ValueCodec for StcTransaction {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl TransactionStore for StcTransactionStorage {
    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<StcTransaction>> {
        self.get(txn_hash)
    }

    fn save_transaction(&self, txn_info: StcTransaction) -> Result<()> {
        self.put(txn_info.id(), txn_info)
    }

    fn save_transaction_batch(&self, txn_vec: Vec<StcTransaction>) -> Result<()> {
        let batch =
            CodecWriteBatch::new_puts(txn_vec.into_iter().map(|txn| (txn.id(), txn)).collect());
        self.write_batch(batch)
    }

    fn get_transactions(
        &self,
        txn_hash_vec: Vec<HashValue>,
    ) -> Result<Vec<Option<StcTransaction>>> {
        self.multiple_get(txn_hash_vec)
    }
}

#[cfg(test)]
mod test;
