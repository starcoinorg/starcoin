// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crate::memory_storage::MemoryStorage;

use crypto::{hash::CryptoHash, HashValue};

use crate::StarcoinStorage;
use std::sync::Arc;
use types::transaction::TransactionInfo;
use types::vm_error::StatusCode;

#[test]
fn test_storage() {
    let store = Arc::new(MemoryStorage::new());
    let storage = StarcoinStorage::new(store).unwrap();
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
        .save(transaction_info1.clone())
        .unwrap();
    let transaction_info2 = storage.transaction_info_store.get(id).unwrap();
    assert!(transaction_info2.is_some());
    assert_eq!(transaction_info1, transaction_info2.unwrap());
}
