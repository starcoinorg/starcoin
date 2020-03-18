// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecStorage, ColumnFamilyName, Repository, ValueCodec};
use anyhow::Result;
use crypto::hash::CryptoHash;
use crypto::HashValue;
use scs::SCSCodec;
use std::sync::Arc;
use types::transaction::TransactionInfo;

pub const TRANSACTION_KEY_NAME: ColumnFamilyName = "transaction_info";
pub struct TransactionInfoStore {
    store: CodecStorage<HashValue, TransactionInfo>,
}

impl ValueCodec for TransactionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl TransactionInfoStore {
    pub fn new(kv_store: Arc<dyn Repository>) -> Self {
        TransactionInfoStore {
            store: CodecStorage::new(kv_store),
        }
    }

    pub fn save(&self, txn_info: TransactionInfo) -> Result<()> {
        self.store.put(txn_info.crypto_hash(), txn_info)
    }

    pub fn get(&self, hash_value: HashValue) -> Result<Option<TransactionInfo>> {
        self.store.get(hash_value)
    }
}
