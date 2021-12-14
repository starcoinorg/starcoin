// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{CodecKVStore, InnerStore, StorageInstance, ValueCodec};
use crate::transaction_info::{BlockTransactionInfo, OldTransactionInfoStorage};
use crate::{
    BlockInfoStore, BlockStore, BlockTransactionInfoStore, Storage, StorageVersion,
    TransactionStore, DEFAULT_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME,
    TRANSACTION_INFO_PREFIX_NAME_V2,
};
use anyhow::Result;
use crypto::HashValue;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_config::RocksdbConfig;
use starcoin_types::block::{Block, BlockBody, BlockHeader, BlockInfo};
use starcoin_types::transaction::{
    RichTransactionInfo, SignedUserTransaction, Transaction, TransactionInfo,
};
use starcoin_types::vm_error::KeptVMStatus;
use std::path::Path;

#[test]
fn test_reopen() {
    let tmpdir = starcoin_config::temp_path();
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
    let tmpdir = starcoin_config::temp_path();
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
    let tmpdir = starcoin_config::temp_path();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
    .unwrap();
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
    let tmpdir = starcoin_config::temp_path();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
    .unwrap();
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
    let tmpdir = starcoin_config::temp_path();
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    );
    let cache_storage = instance.cache().unwrap();
    let db_storage = instance.db().unwrap();
    let storage = Storage::new(instance.clone()).unwrap();

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
    storage
        .transaction_info_storage
        .put(id, transaction_info1.clone())
        .unwrap();
    let transaction_info2 = storage.transaction_info_storage.get(id).unwrap();
    assert!(transaction_info2.is_some());
    assert_eq!(transaction_info1, transaction_info2.unwrap());
    //verfiy cache storage
    let value3 = cache_storage
        .get(TRANSACTION_INFO_PREFIX_NAME_V2, id.to_vec())
        .unwrap()
        .unwrap();
    let transation_info3 = RichTransactionInfo::decode_value(&value3).unwrap();
    assert_eq!(transation_info3, transaction_info1);
    // // verify db storage
    let value4 = db_storage
        .get(TRANSACTION_INFO_PREFIX_NAME_V2, id.to_vec())
        .unwrap()
        .unwrap();
    let transaction_info4 = RichTransactionInfo::decode_value(&value4).unwrap();
    assert_eq!(transaction_info4, transaction_info1);
    // // test remove
    storage.transaction_info_storage.remove(id).unwrap();
    let transaction_info5 = storage.transaction_info_storage.get(id).unwrap();
    assert_eq!(transaction_info5, None);
    // verify cache storage is null
    let value6 = cache_storage
        .get(TRANSACTION_INFO_PREFIX_NAME_V2, id.to_vec())
        .unwrap();
    assert!(value6.is_none());
    let value7 = db_storage
        .get(TRANSACTION_INFO_PREFIX_NAME_V2, id.to_vec())
        .unwrap();
    assert_eq!(value7, None);
}

#[test]
fn test_two_level_storage_read_through() -> Result<()> {
    let tmpdir = starcoin_config::temp_path();

    let transaction_info1 = RichTransactionInfo::new(
        HashValue::random(),
        1,
        TransactionInfo::new(
            HashValue::random(),
            HashValue::zero(),
            vec![].as_slice(),
            0,
            KeptVMStatus::Executed,
        ),
        1,
        1,
    );
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
    let tmpdir = starcoin_config::temp_path();
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

fn generate_old_db(path: &Path) -> Result<Vec<HashValue>> {
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(path, RocksdbConfig::default(), None)?,
    );
    let storage = Storage::new(instance.clone())?;
    let old_transaction_info_storage = OldTransactionInfoStorage::new(instance);

    let block_header = BlockHeader::random();
    let txn = SignedUserTransaction::mock();
    let block = Block::new(
        block_header.clone(),
        BlockBody::new(vec![txn.clone()], None),
    );
    let mut txn_inf_ids = vec![];
    let block_metadata = block.to_metadata(0);
    let txn_info_0 = TransactionInfo::new(
        block_metadata.id(),
        HashValue::random(),
        vec![].as_slice(),
        0,
        KeptVMStatus::Executed,
    );
    storage
        .transaction_storage
        .save_transaction(Transaction::BlockMetadata(block_metadata))?;
    txn_inf_ids.push(txn_info_0.id());
    let txn_info_1 = TransactionInfo::new(
        txn.id(),
        HashValue::random(),
        vec![].as_slice(),
        100,
        KeptVMStatus::Executed,
    );
    txn_inf_ids.push(txn_info_1.id());
    let block_info = BlockInfo::new(
        block_header.id(),
        0.into(),
        AccumulatorInfo::new(HashValue::random(), vec![], 2, 3),
        AccumulatorInfo::new(HashValue::random(), vec![], 1, 1),
    );
    storage
        .transaction_storage
        .save_transaction(Transaction::UserTransaction(txn))?;
    storage.commit_block(block)?;
    storage.save_block_info(block_info)?;

    old_transaction_info_storage.put(
        txn_info_0.id(),
        BlockTransactionInfo {
            block_id: block_header.id(),
            txn_info: txn_info_0,
        },
    )?;
    old_transaction_info_storage.put(
        txn_info_1.id(),
        BlockTransactionInfo {
            block_id: block_header.id(),
            txn_info: txn_info_1,
        },
    )?;

    Ok(txn_inf_ids)
}

#[stest::test]
pub fn test_db_upgrade() -> Result<()> {
    let tmpdir = starcoin_config::temp_path();
    let txn_info_ids = generate_old_db(tmpdir.path())?;
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None)?,
    );
    let storage = Storage::new(instance.clone())?;
    let old_transaction_info_storage = OldTransactionInfoStorage::new(instance);

    let storage = storage.check_upgrade()?;
    for txn_info_id in txn_info_ids {
        assert!(
            old_transaction_info_storage.get(txn_info_id)?.is_none(),
            "expect BlockTransactionInfo is none"
        );
        assert!(
            storage.get_transaction_info(txn_info_id)?.is_some(),
            "expect RichTransactionInfo is some"
        );
    }
    Ok(())
}
