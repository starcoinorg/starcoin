// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{CodecKVStore, InnerStore, StorageInstance, ValueCodec};
use crate::table_info::TableInfoStore;
use crate::tests::{random_txn_info, random_txn_info2};
use crate::{
    BlockStore, BlockTransactionInfoStore, Storage, StorageVersion, DEFAULT_PREFIX_NAME,
    TRANSACTION_INFO_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME_V3,
};
use anyhow::Result;
use starcoin_config::RocksdbConfig;
use starcoin_crypto::HashValue;
use starcoin_types::transaction::StcRichTransactionInfo;
use starcoin_types::{
    account_address::AccountAddress, language_storage::TypeTag, startup_info::SnapshotRange,
};
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};

#[test]
fn test_reopen() {
    let tmpdir = starcoin_config::temp_dir();
    let key = HashValue::random();
    let value = HashValue::zero();
    {
        let db = DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap();
        db.put(DEFAULT_PREFIX_NAME, key.to_vec(), value.to_vec())
            .unwrap();
        assert_eq!(
            db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap(),
            Some(value.to_vec())
        );
    }
    {
        let db = DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap();
        assert_eq!(
            db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap(),
            Some(value.to_vec())
        );
    }
}

#[test]
fn test_open_read_only() {
    let tmpdir = starcoin_config::temp_dir();
    let db = DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap();
    let key = HashValue::random();
    let value = HashValue::zero();
    let result = db.put(DEFAULT_PREFIX_NAME, key.to_vec(), value.to_vec());
    assert!(result.is_ok());
    let path = tmpdir.as_ref().join("starcoindb");
    let db = DBStorage::open_with_cfs(
        path,
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        RocksdbConfig::default(),
        None,
    )
    .unwrap();
    let result = db.put(DEFAULT_PREFIX_NAME, key.to_vec(), value.to_vec());
    assert!(result.is_err());
    let result = db.get(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap();
    assert_eq!(result, Some(value.to_vec()));
}

#[test]
fn test_storage() {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
    .unwrap();
    let transaction_info1 = random_txn_info(0);
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
fn test_iter() {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
    .unwrap();
    let transaction_info1 = random_txn_info(0);
    let id = transaction_info1.id();
    storage
        .transaction_info_storage
        .put(id, transaction_info1.clone())
        .unwrap();
    let mut iter = storage.transaction_info_storage.iter().unwrap();
    iter.seek_to_first();
    let transaction_info2 = iter.next().and_then(|item| item.ok());
    assert!(transaction_info2.is_some());
    assert_eq!(transaction_info1, transaction_info2.unwrap().1);
    assert!(iter.next().is_none());
}

#[test]
fn test_two_level_storage() {
    let tmpdir = starcoin_config::temp_dir();
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    );
    let cache_storage = instance.cache().unwrap();
    let db_storage = instance.db().unwrap();
    let storage = Storage::new(instance.clone()).unwrap();

    let transaction_info1 = random_txn_info(0);
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
        .get(TRANSACTION_INFO_PREFIX_NAME_V3, id.to_vec())
        .unwrap()
        .unwrap();
    let transaction_info3 = StcRichTransactionInfo::decode_value(&value3).unwrap();
    assert_eq!(transaction_info3, transaction_info1);
    // // verify db storage
    let value4 = db_storage
        .get(TRANSACTION_INFO_PREFIX_NAME_V3, id.to_vec())
        .unwrap()
        .unwrap();
    let transaction_info4 = StcRichTransactionInfo::decode_value(&value4).unwrap();
    assert_eq!(transaction_info4, transaction_info1);
    // // test remove
    storage.transaction_info_storage.remove(id).unwrap();
    let transaction_info5 = storage.transaction_info_storage.get(id).unwrap();
    assert_eq!(transaction_info5, None);
    // verify cache storage is null
    let value6 = cache_storage
        .get(TRANSACTION_INFO_PREFIX_NAME_V3, id.to_vec())
        .unwrap();
    assert!(value6.is_none());
    let value7 = db_storage
        .get(TRANSACTION_INFO_PREFIX_NAME_V3, id.to_vec())
        .unwrap();
    assert_eq!(value7, None);
}

#[test]
fn test_two_level_storage_read_through() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();

    let transaction_info1 = random_txn_info2(1, 0);
    let id = transaction_info1.id();

    {
        let storage = Storage::new(StorageInstance::new_db_instance(
            DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
        ))
        .unwrap();
        storage
            .transaction_info_storage
            .put(id, transaction_info1.clone())
            .unwrap();
    }
    let storage_instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    );
    let storage2 = Storage::new(storage_instance.clone()).unwrap();

    let transaction_info2 = storage2.transaction_info_storage.get(id).unwrap();
    assert_eq!(transaction_info1, transaction_info2.unwrap());

    //verfiy cache storage get null
    let transaction_info_data = storage_instance
        .cache()
        .unwrap()
        .get(TRANSACTION_INFO_PREFIX_NAME, id.to_vec())?;
    assert!(transaction_info_data.is_none());

    //let transaction_info3 =
    //BlockTransactionInfo::decode_value(&transaction_info_data.unwrap()).unwrap();
    //assert_eq!(transaction_info3, transaction_info1);
    Ok(())
}

#[test]
fn test_missing_key_handle() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    );
    let cache_storage = instance.cache().unwrap();
    let db_storage = instance.db().unwrap();
    let storage = Storage::new(instance.clone()).unwrap();
    let key = HashValue::random();
    let result = storage.get_transaction_info(key)?;
    assert!(result.is_none());
    let value2 = cache_storage.get(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec())?;
    assert!(value2.is_none());
    let value3 = db_storage.get(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec())?;
    assert!(value3.is_none());
    // test remove
    let result2 = instance.remove(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec());
    assert!(result2.is_ok());
    let value4 = cache_storage.get(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec())?;
    assert!(value4.is_none());
    let contains = instance.contains_key(TRANSACTION_INFO_PREFIX_NAME, key.clone().to_vec())?;
    assert!(!contains);
    Ok(())
}

#[test]
pub fn test_snapshot_range() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None)?,
    );
    let storage = Storage::new(instance)?;
    let snapshot_range = storage.get_snapshot_range()?;
    assert!(snapshot_range.is_none(), "export snapshot_range is none");
    let snapshot_range = SnapshotRange::new(1, 1000);
    storage.save_snapshot_range(snapshot_range)?;
    let snapshot_range = storage.get_snapshot_range()?;
    assert!(snapshot_range.is_some(), "expect snapshot_range is some");
    let snapshot_range = snapshot_range.unwrap();
    assert_eq!(snapshot_range.get_start(), 1);
    assert_eq!(snapshot_range.get_end(), 1000);
    Ok(())
}

#[test]
pub fn test_cache_evict_multi_get() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new_with_capacity(2, None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None)?,
    );
    let storage = Storage::new(instance.clone())?;
    let transaction_info1 = random_txn_info(0);
    let id1 = transaction_info1.id();

    let transaction_info2 = random_txn_info(0);
    let id2 = transaction_info2.id();

    let transaction_info3 = random_txn_info(0);
    let id3 = transaction_info3.id();
    storage
        .transaction_info_storage
        .put(id1, transaction_info1.clone())?;
    storage
        .transaction_info_storage
        .put(id2, transaction_info2.clone())?;
    storage
        .transaction_info_storage
        .put(id3, transaction_info3.clone())?;
    let cache_storage = instance.cache().unwrap();
    let cache_infos = cache_storage.multi_get(
        TRANSACTION_INFO_PREFIX_NAME_V3,
        vec![id1.to_vec(), id2.to_vec(), id3.to_vec()],
    )?;
    assert!(&cache_infos.first().unwrap().is_none(), "id1 has evicted");
    assert_eq!(
        StcRichTransactionInfo::decode_value(&cache_infos.get(1).unwrap().clone().unwrap())?,
        transaction_info2
    );
    assert_eq!(
        StcRichTransactionInfo::decode_value(&cache_infos.get(2).unwrap().clone().unwrap())?,
        transaction_info3
    );
    let infos = storage
        .transaction_info_storage
        .multiple_get(vec![id1, id2, id3])?;
    assert_eq!(infos.first().unwrap().clone().unwrap(), transaction_info1);
    assert_eq!(infos.get(1).unwrap().clone().unwrap(), transaction_info2);
    assert_eq!(infos.get(2).unwrap().clone().unwrap(), transaction_info3);
    Ok(())
}

#[test]
fn test_table_info_storage() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None)?,
    );
    let storage = Storage::new(instance)?;
    let key1 = TableHandle(AccountAddress::random()).into();
    let table_info1 = TableInfo::new(TypeTag::U8, TypeTag::U8);
    storage.save_table_info(key1, table_info1.clone().into())?;
    let val = storage.get_table_info(key1);
    assert!(val.is_ok());
    assert_eq!(val.unwrap().unwrap(), table_info1.into());
    let key2 = TableHandle(AccountAddress::random()).into();
    let val = storage.get_table_info(key2);
    assert!(val.is_ok());
    assert_eq!(val.unwrap(), None);
    let keys = vec![
        TableHandle(AccountAddress::random()).into(),
        TableHandle(AccountAddress::random()).into(),
    ];
    let vals = vec![
        TableInfo::new(TypeTag::U8, TypeTag::Address).into(),
        TableInfo::new(TypeTag::Address, TypeTag::U128).into(),
    ];
    let table_infos = keys
        .clone()
        .into_iter()
        .zip(vals.clone())
        .collect::<Vec<_>>();
    storage.save_table_infos(table_infos)?;
    let vals2 = storage.get_table_infos(keys);
    assert!(vals2.is_ok());
    let vals2 = vals2
        .unwrap()
        .into_iter()
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();
    assert_eq!(vals, vals2);
    Ok(())
}
