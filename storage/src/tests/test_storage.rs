// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crypto::{hash::PlainCryptoHash, HashValue};

use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{InnerStore, StorageInstance, ValueCodec};
use crate::{Storage, TRANSACTION_INFO_PREFIX_NAME};
use anyhow::Result;
use starcoin_types::transaction::TransactionInfo;
use starcoin_types::vm_error::StatusCode;
use std::sync::Arc;

#[test]
fn test_storage() {
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = libra_temppath::TempPath::new();
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
    let tmpdir = libra_temppath::TempPath::new();
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
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_db_instance(db_storage.clone())).unwrap();

    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        HashValue::zero(),
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
        db_storage.clone(),
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
