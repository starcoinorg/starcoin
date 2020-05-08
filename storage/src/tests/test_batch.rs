// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{InnerStore, ValueCodec};
use crate::DEFAULT_PREFIX_NAME;
use crypto::{hash::CryptoHash, HashValue};
use starcoin_types::transaction::TransactionInfo;
use starcoin_types::vm_error::StatusCode;
use std::sync::Arc;

#[test]
fn test_db_batch() {
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let mut write_batch = WriteBatch::new();
    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        StatusCode::ABORTED,
    );
    let id = transaction_info1.clone().crypto_hash();
    write_batch
        .put::<HashValue, TransactionInfo>(id, transaction_info1.clone())
        .unwrap();
    let transaction_info2 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        HashValue::zero(),
        1,
        StatusCode::ABORTED,
    );
    let id2 = transaction_info2.clone().crypto_hash();
    write_batch
        .put::<HashValue, TransactionInfo>(id2, transaction_info2.clone())
        .unwrap();
    db_storage.write_batch(write_batch).unwrap();
    assert_eq!(
        TransactionInfo::decode_value(
            &db_storage
                .get(DEFAULT_PREFIX_NAME, id.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info1.clone()
    );
    assert_eq!(
        TransactionInfo::decode_value(
            &db_storage
                .get(DEFAULT_PREFIX_NAME, id2.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info2.clone()
    );
}

#[test]
fn test_cache_batch() {
    let cache_storage = Arc::new(CacheStorage::new());
    let mut write_batch = WriteBatch::new();
    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        StatusCode::ABORTED,
    );
    let id = transaction_info1.clone().crypto_hash();
    write_batch
        .put::<HashValue, TransactionInfo>(id, transaction_info1.clone())
        .unwrap();
    let transaction_info2 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        HashValue::zero(),
        1,
        StatusCode::ABORTED,
    );
    let id2 = transaction_info2.clone().crypto_hash();
    write_batch
        .put::<HashValue, TransactionInfo>(id2, transaction_info2.clone())
        .unwrap();
    cache_storage.write_batch(write_batch).unwrap();
    assert_eq!(
        TransactionInfo::decode_value(
            &cache_storage
                .get(DEFAULT_PREFIX_NAME, id.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info1.clone()
    );
    assert_eq!(
        TransactionInfo::decode_value(
            &cache_storage
                .get(DEFAULT_PREFIX_NAME, id2.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info2.clone()
    );
}
