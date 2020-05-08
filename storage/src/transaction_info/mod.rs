// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::storage::{CodecStorage, ValueCodec};
use crate::TRANSACTION_INFO_PREFIX_NAME;
use crate::{define_storage, TransactionInfoStore};
use anyhow::{Error, Result};
use crypto::HashValue;
use scs::SCSCodec;
use starcoin_types::transaction::TransactionInfo;
use std::sync::Arc;

define_storage!(
    TransactionInfoStorage,
    HashValue,
    TransactionInfo,
    TRANSACTION_INFO_PREFIX_NAME
);

impl ValueCodec for TransactionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl TransactionInfoStore for TransactionInfoStorage {
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>, Error> {
        self.store.get(txn_hash)
    }

    fn save_transaction_info(&self, txn_info: TransactionInfo) -> Result<(), Error> {
        self.store.put(txn_info.transaction_hash(), txn_info)
    }

    fn save_transaction_infos(&self, vec_txn_info: Vec<TransactionInfo>) -> Result<(), Error> {
        let mut batch = WriteBatch::new_with_name(TRANSACTION_INFO_PREFIX_NAME);
        for txn_info in vec_txn_info {
            batch.put(txn_info.transaction_hash(), txn_info)?;
        }
        self.store.write_batch(batch)
    }
}
