// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{CodecWriteBatch, InnerStore, ValueCodec};
use crate::{DEFAULT_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME_V2};
use anyhow::Result;
use starcoin_config::RocksdbConfig;
use starcoin_crypto::HashValue;
use starcoin_types::transaction::RichTransactionInfo;
use std::convert::TryInto;
use std::sync::Arc;

#[test]
fn test_db_batch() {
    let tmpdir = starcoin_config::temp_dir();
    let db_storage =
        Arc::new(DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap());
    let mut write_batch = CodecWriteBatch::new();
    let transaction_info1 = RichTransactionInfo::random();
    let id = transaction_info1.id();
    write_batch.put(id, transaction_info1.clone()).unwrap();
    let transaction_info2 = RichTransactionInfo::random();
    let id2 = transaction_info2.id();
    write_batch.put(id2, transaction_info2.clone()).unwrap();
    db_storage
        .write_batch(DEFAULT_PREFIX_NAME, write_batch.try_into().unwrap())
        .unwrap();
    assert_eq!(
        RichTransactionInfo::decode_value(
            &db_storage
                .get_raw(DEFAULT_PREFIX_NAME, id.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info1
    );
    assert_eq!(
        RichTransactionInfo::decode_value(
            &db_storage
                .get_raw(DEFAULT_PREFIX_NAME, id2.to_vec())
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
    let transaction_info1 = RichTransactionInfo::random();
    let id = transaction_info1.id();
    write_batch.put(id, transaction_info1.clone()).unwrap();
    let transaction_info2 = RichTransactionInfo::random();
    let id2 = transaction_info2.id();
    write_batch.put(id2, transaction_info2.clone()).unwrap();
    cache_storage
        .write_batch(DEFAULT_PREFIX_NAME, write_batch.try_into().unwrap())
        .unwrap();
    assert_eq!(
        RichTransactionInfo::decode_value(
            &cache_storage
                .get_raw(DEFAULT_PREFIX_NAME, id.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info1
    );
    assert_eq!(
        RichTransactionInfo::decode_value(
            &cache_storage
                .get_raw(DEFAULT_PREFIX_NAME, id2.to_vec())
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        transaction_info2
    );
}

#[test]
fn test_batch_comm() {
    let tmpdir = starcoin_config::temp_dir();
    let key = HashValue::random();
    let value = HashValue::zero();
    let db = DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap();
    let mut write_batch = WriteBatch::new();
    write_batch.put(key.to_vec(), value.to_vec()).unwrap();
    write_batch.delete(key.to_vec()).unwrap();
    let result = db.write_batch(DEFAULT_PREFIX_NAME, write_batch.clone());
    assert!(result.is_ok());
    let result = db.get_raw(DEFAULT_PREFIX_NAME, key.to_vec()).unwrap();
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

#[test]
fn test_write_batch_multi_get() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let db_storage =
        Arc::new(DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap());
    let mut write_batch = CodecWriteBatch::new();
    let transaction_info1 = RichTransactionInfo::random();
    let id1 = transaction_info1.id();
    write_batch.put(id1, transaction_info1.clone())?;
    let transaction_info2 = RichTransactionInfo::random();
    let id2 = transaction_info2.id();
    write_batch.put(id2, transaction_info2.clone())?;
    db_storage.write_batch(TRANSACTION_INFO_PREFIX_NAME_V2, write_batch.try_into()?)?;

    let infos = db_storage.multi_get(
        TRANSACTION_INFO_PREFIX_NAME_V2,
        vec![id1.to_vec(), id2.to_vec()],
    )?;
    assert_eq!(
        RichTransactionInfo::decode_value(&infos.get(0).unwrap().clone().unwrap())?,
        transaction_info1
    );
    assert_eq!(
        RichTransactionInfo::decode_value(&infos.get(1).unwrap().clone().unwrap())?,
        transaction_info2
    );
    Ok(())
}

#[test]
fn test_cache_multi_get_no_evict() -> Result<()> {
    let cache_storage = Arc::new(CacheStorage::new(None));
    let mut write_batch = CodecWriteBatch::new();
    let transaction_info1 = RichTransactionInfo::random();
    let id1 = transaction_info1.id();
    write_batch.put(id1, transaction_info1.clone())?;
    let transaction_info2 = RichTransactionInfo::random();
    let id2 = transaction_info2.id();
    write_batch.put(id2, transaction_info2.clone())?;
    cache_storage.write_batch(TRANSACTION_INFO_PREFIX_NAME_V2, write_batch.try_into()?)?;

    let infos = cache_storage.multi_get(
        TRANSACTION_INFO_PREFIX_NAME_V2,
        vec![id1.to_vec(), id2.to_vec()],
    )?;

    assert_eq!(
        RichTransactionInfo::decode_value(&infos.get(0).unwrap().clone().unwrap())?,
        transaction_info1
    );
    assert_eq!(
        RichTransactionInfo::decode_value(&infos.get(1).unwrap().clone().unwrap())?,
        transaction_info2
    );
    Ok(())
}

#[test]
fn test_cache_multi_get_with_evict() -> Result<()> {
    let cache_storage = Arc::new(CacheStorage::new_with_capacity(2, None));
    let mut write_batch = CodecWriteBatch::new();
    let transaction_info1 = RichTransactionInfo::random();
    let id1 = transaction_info1.id();
    write_batch.put(id1, transaction_info1)?;
    let transaction_info2 = RichTransactionInfo::random();
    let id2 = transaction_info2.id();
    write_batch.put(id2, transaction_info2.clone())?;
    let transaction_info3 = RichTransactionInfo::random();
    let id3 = transaction_info3.id();
    write_batch.put(id3, transaction_info3.clone())?;
    cache_storage.write_batch(TRANSACTION_INFO_PREFIX_NAME_V2, write_batch.try_into()?)?;

    let infos = cache_storage.multi_get(
        TRANSACTION_INFO_PREFIX_NAME_V2,
        vec![id1.to_vec(), id2.to_vec(), id3.to_vec()],
    )?;

    assert!(&infos.get(0).unwrap().is_none(), "id1 has evicted");
    assert_eq!(
        RichTransactionInfo::decode_value(&infos.get(1).unwrap().clone().unwrap())?,
        transaction_info2
    );
    assert_eq!(
        RichTransactionInfo::decode_value(&infos.get(2).unwrap().clone().unwrap())?,
        transaction_info3
    );
    Ok(())
}

#[test]
fn test_batch_comm_multi_get() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let db = DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None)?;
    let mut write_batch = WriteBatch::new();
    let mut key_vec = vec![];
    let mut value_vec = vec![];
    for _i in 0..100 {
        let key = HashValue::random();
        let value = HashValue::random();
        key_vec.push(key.to_vec());
        value_vec.push(value);
        write_batch.put(key.to_vec(), value.to_vec())?;
    }
    db.write_batch(DEFAULT_PREFIX_NAME, write_batch)?;
    let mut delete_batch = WriteBatch::new();
    for i in 51..100 {
        delete_batch.delete(key_vec.get(i).unwrap().to_vec())?;
    }
    db.write_batch(DEFAULT_PREFIX_NAME, delete_batch)?;

    let values = db.multi_get(DEFAULT_PREFIX_NAME, key_vec)?;
    for i in 0..51 {
        assert_eq!(
            HashValue::decode_value(&values.get(i).unwrap().clone().unwrap())?,
            value_vec.get(i).unwrap().clone()
        )
    }
    for i in 51..100 {
        assert!(values.get(i).unwrap().is_none(), "values should be none");
    }
    Ok(())
}
