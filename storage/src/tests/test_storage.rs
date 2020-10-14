// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{CodecKVStore, InnerStore, StorageInstance, ValueCodec, CACHE_NONE_OBJECT};
use crate::{
    Storage, TransactionInfoStore, DEFAULT_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME,
    VEC_PREFIX_NAME,
};
use anyhow::Result;
use crypto::HashValue;
use starcoin_types::transaction::TransactionInfo;
use starcoin_types::vm_error::KeptVMStatus;

#[test]
fn test_reopen() {
    let tmpdir = starcoin_config::temp_path();
    let key = HashValue::random();
    let value = HashValue::zero();
    {
        let db = DBStorage::new(tmpdir.path()).unwrap();
        db.put(DEFAULT_PREFIX_NAME, key.to_vec(), value.to_vec())
            .unwrap();
        assert_eq!(
            db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap(),
            Some(value.to_vec())
        );
    }
    {
        let db = DBStorage::new(tmpdir.path()).unwrap();
        assert_eq!(
            db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap(),
            Some(value.to_vec())
        );
    }
}

#[test]
fn test_open_read_only() {
    let tmpdir = starcoin_config::temp_path();
    let db = DBStorage::new(tmpdir.path()).unwrap();
    let key = HashValue::random();
    let value = HashValue::zero();
    let result = db.put(DEFAULT_PREFIX_NAME, key.to_vec(), value.to_vec());
    assert!(result.is_ok());
    let path = tmpdir.as_ref().join("starcoindb");
    let db = DBStorage::open_with_cfs(path, VEC_PREFIX_NAME.to_vec(), true).unwrap();
    let result = db.put(DEFAULT_PREFIX_NAME, key.to_vec(), value.to_vec());
    assert!(result.is_err());
    let result = db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap();
    assert_eq!(result, Some(value.to_vec()));
}

#[test]
fn test_storage() {
    let tmpdir = starcoin_config::temp_path();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(),
        DBStorage::new(tmpdir.path()).unwrap(),
    ))
    .unwrap();
    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        vec![].as_slice(),
        0,
        KeptVMStatus::Executed,
    );
    let id = transaction_info1.id();
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
    let tmpdir = starcoin_config::temp_path();
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(),
        DBStorage::new(tmpdir.path()).unwrap(),
    );
    let cache_storage = instance.cache().unwrap();
    let db_storage = instance.db().unwrap();
    let storage = Storage::new(instance).unwrap();

    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        vec![].as_slice(),
        0,
        KeptVMStatus::Executed,
    );
    let id = transaction_info1.id();
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
        .get_obj(TRANSACTION_INFO_PREFIX_NAME, id.to_vec())
        .unwrap();
    assert_eq!(value6.unwrap(), CACHE_NONE_OBJECT.clone());
    let value7 = db_storage
        .get(TRANSACTION_INFO_PREFIX_NAME, id.to_vec())
        .unwrap();
    assert_eq!(value7, None);
}

#[test]
fn test_two_level_storage_read_through() -> Result<()> {
    let tmpdir = starcoin_config::temp_path();

    let transaction_info1 = TransactionInfo::new(
        HashValue::random(),
        HashValue::zero(),
        vec![].as_slice(),
        0,
        KeptVMStatus::Executed,
    );
    let id = transaction_info1.id();

    {
        let storage = Storage::new(StorageInstance::new_db_instance(
            DBStorage::new(tmpdir.path()).unwrap(),
        ))
        .unwrap();
        storage
            .transaction_info_storage
            .put(id, transaction_info1.clone())
            .unwrap();
    }
    let storage_instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(),
        DBStorage::new(tmpdir.path()).unwrap(),
    );
    let storage2 = Storage::new(storage_instance.clone()).unwrap();

    let transaction_info2 = storage2.transaction_info_storage.get(id).unwrap();
    assert_eq!(transaction_info1, transaction_info2.unwrap());

    //verfiy cache storage
    let transaction_info_data = storage_instance
        .cache()
        .unwrap()
        .get(TRANSACTION_INFO_PREFIX_NAME, id.to_vec())?;
    let transaction_info3 = TransactionInfo::decode_value(&transaction_info_data.unwrap()).unwrap();
    assert_eq!(transaction_info3, transaction_info1);
    Ok(())
}

#[test]
fn test_missing_key_handle() -> Result<()> {
    let tmpdir = starcoin_config::temp_path();
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(),
        DBStorage::new(tmpdir.path()).unwrap(),
    );
    let cache_storage = instance.cache().unwrap();
    let db_storage = instance.db().unwrap();
    let storage = Storage::new(instance.clone()).unwrap();
    let key = HashValue::random();
    let result = storage.get_transaction_info(key)?;
    assert!(result.is_none());
    let value2 = cache_storage.get_obj(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec())?;
    assert_eq!(value2.unwrap(), CACHE_NONE_OBJECT.clone());
    let value3 = db_storage.get(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec())?;
    assert!(value3.is_none());
    // test remove
    let result2 = instance.remove(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec());
    assert!(result2.is_ok());
    let value4 = cache_storage.get(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec())?;
    assert!(value4.is_none());
    let contains = instance.contains_key(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec())?;
    assert_eq!(contains, false);
    Ok(())
}
