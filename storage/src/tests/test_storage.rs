// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crypto::{hash::PlainCryptoHash, HashValue};

use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{InnerStore, StorageInstance, ValueCodec};
use crate::{Storage, DEFAULT_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME};
use anyhow::Result;
use starcoin_types::transaction::TransactionInfo;
use starcoin_types::vm_error::StatusCode;
use std::sync::Arc;

#[test]
fn test_reopen() {
    let tmpdir = starcoin_config::temp_path();
    let key = HashValue::random();
    let value = HashValue::zero();
    {
        let db = DBStorage::new(tmpdir.path());
        db.put(DEFAULT_PREFIX_NAME, key.to_vec(), value.to_vec())
            .unwrap();
        assert_eq!(
            db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap(),
            Some(value.to_vec())
        );
    }
    {
        let db = DBStorage::new(tmpdir.path());
        assert_eq!(
            db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap(),
            Some(value.to_vec())
        );
    }
}

#[test]
fn test_open_read_only() {
    let tmpdir = starcoin_config::temp_path();
    let db = DBStorage::new(tmpdir.path());
    let key = HashValue::random();
    let value = HashValue::zero();
    let result = db.put(DEFAULT_PREFIX_NAME, key.to_vec(), value.to_vec());
    assert!(result.is_ok());
    let path = tmpdir.as_ref().join("starcoindb");
    let db = DBStorage::open(path, true).unwrap();
    let result = db.put(DEFAULT_PREFIX_NAME, key.to_vec(), value.to_vec());
    assert!(result.is_err());
    let result = db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap();
    assert_eq!(result, Some(value.to_vec()));
}

#[test]
fn test_storage() {
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = starcoin_config::temp_path();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage,
        db_storage,
    ))
    .unwrap();
    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        HashValue::zero(),
        vec![],
        0,
        StatusCode::ABORTED,
    );
    let id = transaction_info1.crypto_hash();
    storage
        .transaction_info_storage
        .put(id, transaction_info1.clone())
        .unwrap();
    let transaction_info2 = storage.transaction_info_storage.get(id).unwrap();
    assert!(transaction_info2.is_some());
    assert_eq!(transaction_info1, transaction_info2.unwrap());
}
#[test]
fn test_two_level_storage() {
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = starcoin_config::temp_path();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage.clone(),
        db_storage.clone(),
    ))
    .unwrap();

    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        HashValue::zero(),
        vec![],
        0,
        StatusCode::ABORTED,
    );
    let id = transaction_info1.crypto_hash();
    storage
        .transaction_info_storage
        .put(id, transaction_info1.clone())
        .unwrap();
    let transaction_info2 = storage.transaction_info_storage.get(id).unwrap();
    assert!(transaction_info2.is_some());
    assert_eq!(transaction_info1, transaction_info2.unwrap());
    //verfiy cache storage
    let value3 = cache_storage
        .get(TRANSACTION_INFO_PREFIX_NAME, id.to_vec())
        .unwrap()
        .unwrap();
    let transation_info3 = TransactionInfo::decode_value(&value3).unwrap();
    assert_eq!(transation_info3, transaction_info1);
    // // verify db storage
    let value4 = db_storage
        .get(TRANSACTION_INFO_PREFIX_NAME, id.to_vec())
        .unwrap()
        .unwrap();
    let transaction_info4 = TransactionInfo::decode_value(&value4).unwrap();
    assert_eq!(transaction_info4, transaction_info1);
    // // test remove
    storage.transaction_info_storage.remove(id).unwrap();
    let transaction_info5 = storage.transaction_info_storage.get(id).unwrap();
    assert_eq!(transaction_info5, None);
    // verify cache storage is null
    let value6 = cache_storage
        .get(TRANSACTION_INFO_PREFIX_NAME, id.to_vec())
        .unwrap();
    assert_eq!(value6, None);
    let value7 = db_storage
        .get(TRANSACTION_INFO_PREFIX_NAME, id.to_vec())
        .unwrap();
    assert_eq!(value7, None);
}

#[test]
fn test_two_level_storage_read_through() -> Result<()> {
    let tmpdir = starcoin_config::temp_path();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_db_instance(db_storage.clone())).unwrap();

    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        HashValue::zero(),
        vec![],
        0,
        StatusCode::ABORTED,
    );
    let id = transaction_info1.crypto_hash();
    storage
        .transaction_info_storage
        .put(id, transaction_info1.clone())
        .unwrap();

    let cache_storage = Arc::new(CacheStorage::new());
    let storage2 = Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage.clone(),
        db_storage,
    ))
    .unwrap();

    let transaction_info2 = storage2.transaction_info_storage.get(id).unwrap();
    assert!(transaction_info2.is_some());
    assert_eq!(transaction_info1, transaction_info2.unwrap());

    //verfiy cache storage
    let transaction_info_data = cache_storage.get(TRANSACTION_INFO_PREFIX_NAME, id.to_vec())?;
    assert!(transaction_info_data.is_some());
    let transaction_info3 = TransactionInfo::decode_value(&transaction_info_data.unwrap()).unwrap();
    assert_eq!(transaction_info3, transaction_info1);
    Ok(())
}
