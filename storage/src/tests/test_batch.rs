// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{InnerStore, ValueCodec};
use crate::DEFAULT_PREFIX_NAME;
use crypto::HashValue;
use starcoin_types::transaction::TransactionInfo;
use starcoin_types::vm_error::StatusCode;
use std::sync::Arc;

#[test]
fn test_db_batch() {
    let tmpdir = starcoin_config::temp_path();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let mut write_batch = WriteBatch::new();
    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        vec![].as_slice(),
        0,
        StatusCode::ABORTED,
    );
    let id = transaction_info1.id();
    write_batch
        .put::<HashValue, TransactionInfo>(id, transaction_info1.clone())
        .unwrap();
    let transaction_info2 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        vec![].as_slice(),
        1,
        StatusCode::ABORTED,
    );
    let id2 = transaction_info2.id();
    write_batch
        .put::<HashValue, TransactionInfo>(id2, transaction_info2.clone())
        .unwrap();
    db_storage
        .write_batch(DEFAULT_PREFIX_NAME, write_batch)
        .unwrap();
    assert_eq!(
        TransactionInfo::decode_value(
            &db_storage
                .get(DEFAULT_PREFIX_NAME, id.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info1
    );
    assert_eq!(
        TransactionInfo::decode_value(
            &db_storage
                .get(DEFAULT_PREFIX_NAME, id2.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info2
    );
}

#[test]
fn test_cache_batch() {
    let cache_storage = Arc::new(CacheStorage::new());
    let mut write_batch = WriteBatch::new();
    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        vec![].as_slice(),
        0,
        StatusCode::ABORTED,
    );
    let id = transaction_info1.id();
    write_batch
        .put::<HashValue, TransactionInfo>(id, transaction_info1.clone())
        .unwrap();
    let transaction_info2 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        vec![].as_slice(),
        1,
        StatusCode::ABORTED,
    );
    let id2 = transaction_info2.id();
    write_batch
        .put::<HashValue, TransactionInfo>(id2, transaction_info2.clone())
        .unwrap();
    cache_storage
        .write_batch(DEFAULT_PREFIX_NAME, write_batch)
        .unwrap();
    assert_eq!(
        TransactionInfo::decode_value(
            &cache_storage
                .get(DEFAULT_PREFIX_NAME, id.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info1
    );
    assert_eq!(
        TransactionInfo::decode_value(
            &cache_storage
                .get(DEFAULT_PREFIX_NAME, id2.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info2
    );
}

#[test]
fn test_batch_comm() {
    let tmpdir = starcoin_config::temp_path();
    let key = HashValue::random();
    let value = HashValue::zero();
    let db = DBStorage::new(tmpdir.path());
    let mut write_batch = WriteBatch::new();
    write_batch.put::<HashValue, HashValue>(key, value).unwrap();
    write_batch.delete::<HashValue>(key).unwrap();
    let result = db.write_batch(DEFAULT_PREFIX_NAME, write_batch.clone());
    assert!(result.is_ok());
    let result = db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap();
    assert_eq!(result, None);
    let mut key_vec = vec![];
    write_batch.clone().clear().unwrap();
    let mut new_batch = write_batch.clone();
    for _i in 0..100 {
        let key = HashValue::random();
        key_vec.push(key);
        new_batch.put::<HashValue, HashValue>(key, value).unwrap();
    }
    let result = db.write_batch(DEFAULT_PREFIX_NAME, new_batch);
    assert!(result.is_ok());
    let mut new_batch2 = write_batch;
    for key in key_vec {
        new_batch2.delete::<HashValue>(key).unwrap();
    }
    let result = db.write_batch(DEFAULT_PREFIX_NAME, new_batch2);
    assert!(result.is_ok());
}
