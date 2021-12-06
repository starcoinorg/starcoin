// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{CodecWriteBatch, InnerStore, ValueCodec};
use crate::DEFAULT_PREFIX_NAME;
use crypto::HashValue;
use starcoin_config::RocksdbConfig;
use starcoin_types::transaction::{RichTransactionInfo, TransactionInfo};
use starcoin_types::vm_error::KeptVMStatus;
use std::convert::TryInto;
use std::sync::Arc;

#[test]
fn test_db_batch() {
    let tmpdir = starcoin_config::temp_path();
    let db_storage =
        Arc::new(DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap());
    let mut write_batch = CodecWriteBatch::new();
    let transaction_info1 = RichTransactionInfo::new(
        HashValue::random(),
        rand::random(),
        TransactionInfo::new(
            HashValue::random(),
            HashValue::zero(),
            vec![].as_slice(),
            0,
            KeptVMStatus::Executed,
        ),
        rand::random(),
        rand::random(),
    );
    let id = transaction_info1.id();
    write_batch.put(id, transaction_info1.clone()).unwrap();
    let transaction_info2 = RichTransactionInfo::new(
        HashValue::random(),
        rand::random(),
        TransactionInfo::new(
            HashValue::random(),
            HashValue::zero(),
            vec![].as_slice(),
            1,
            KeptVMStatus::Executed,
        ),
        rand::random(),
        rand::random(),
    );
    let id2 = transaction_info2.id();
    write_batch.put(id2, transaction_info2.clone()).unwrap();
    db_storage
        .write_batch(DEFAULT_PREFIX_NAME, write_batch.try_into().unwrap())
        .unwrap();
    assert_eq!(
        RichTransactionInfo::decode_value(
            &db_storage
                .get(DEFAULT_PREFIX_NAME, id.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info1
    );
    assert_eq!(
        RichTransactionInfo::decode_value(
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
    let cache_storage = Arc::new(CacheStorage::new(None));
    let mut write_batch = CodecWriteBatch::new();
    let transaction_info1 = RichTransactionInfo::new(
        HashValue::random(),
        rand::random(),
        TransactionInfo::new(
            HashValue::random(),
            HashValue::zero(),
            vec![].as_slice(),
            0,
            KeptVMStatus::Executed,
        ),
        rand::random(),
        rand::random(),
    );
    let id = transaction_info1.id();
    write_batch.put(id, transaction_info1.clone()).unwrap();
    let transaction_info2 = RichTransactionInfo::new(
        HashValue::random(),
        rand::random(),
        TransactionInfo::new(
            HashValue::random(),
            HashValue::zero(),
            vec![].as_slice(),
            1,
            KeptVMStatus::Executed,
        ),
        rand::random(),
        rand::random(),
    );
    let id2 = transaction_info2.id();
    write_batch.put(id2, transaction_info2.clone()).unwrap();
    cache_storage
        .write_batch(DEFAULT_PREFIX_NAME, write_batch.try_into().unwrap())
        .unwrap();
    assert_eq!(
        RichTransactionInfo::decode_value(
            &cache_storage
                .get(DEFAULT_PREFIX_NAME, id.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info1
    );
    assert_eq!(
        RichTransactionInfo::decode_value(
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
    let db = DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap();
    let mut write_batch = WriteBatch::new();
    write_batch.put(key.to_vec(), value.to_vec()).unwrap();
    write_batch.delete(key.to_vec()).unwrap();
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
        new_batch.put(key.to_vec(), value.to_vec()).unwrap();
    }
    let result = db.write_batch(DEFAULT_PREFIX_NAME, new_batch);
    assert!(result.is_ok());
    let mut new_batch2 = write_batch;
    for key in key_vec {
        new_batch2.delete(key.to_vec()).unwrap();
    }
    let result = db.write_batch(DEFAULT_PREFIX_NAME, new_batch2);
    assert!(result.is_ok());
}
