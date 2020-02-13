// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::kv::KVStore;
use crate::transaction_info_store::TransactionInfoStore;
use anyhow::Result;
use std::sync::Arc;

pub mod kv;
pub mod transaction_info_store;

struct StarcoinStorage {
    transaction_info_store: TransactionInfoStore,
}

impl StarcoinStorage {
    pub fn new(kv_store: Arc<dyn KVStore>) -> Result<Self> {
        Ok(Self {
            transaction_info_store: TransactionInfoStore::new(kv_store.clone()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kv::HashMapKVStore;
    use anyhow::Result;
    use crypto::{hash::CryptoHash, HashValue};
    use types::transaction::TransactionInfo;
    use types::vm_error::StatusCode;

    #[test]
    fn test_storage() {
        let kv = Arc::new(HashMapKVStore::new());
        let storage = StarcoinStorage::new(kv).unwrap();
        let transaction_info1 = TransactionInfo::new(
            HashValue::random(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            StatusCode::ABORTED,
        );
        let id = transaction_info1.crypto_hash();
        storage
            .transaction_info_store
            .save(transaction_info1.clone());
        let transaction_info2 = storage.transaction_info_store.get(id).unwrap();
        assert!(transaction_info2.is_some());
        assert_eq!(transaction_info1, transaction_info2.unwrap());
    }
}
