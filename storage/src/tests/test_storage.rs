// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crate::batch::{WriteBatch, WriteBatchData, WriteBatchWithColumn};
use crate::block::{
    DagSyncBlock, FailedBlock, OldBlockHeaderStorage, OldBlockInnerStorage, OldFailedBlockStorage,
    OldFailedBlockV2,
};
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::{CodecKVStore, InnerStore, StorageInstance, ValueCodec, WriteOp};
use crate::table_info::TableInfoStore;
use crate::transaction::LegacyTransactionStorage;
use crate::transaction_info::{BlockTransactionInfo, OldTransactionInfoStorage};
use crate::{
    BlockInfoStore, BlockStore, BlockTransactionInfoStore, Storage, StorageVersion,
    BLOCK_PREFIX_NAME, DAG_SYNC_BLOCK_PREFIX_NAME, DEFAULT_PREFIX_NAME,
    TRANSACTION_INFO_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME_V2,
};
use anyhow::{Ok, Result};
use bcs_ext::BCSCodec;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_config::RocksdbConfig;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_types::block::{Block, BlockBody, BlockHeader, BlockHeaderBuilder, BlockInfo};
use starcoin_types::startup_info::SnapshotRange;
use starcoin_types::transaction::{RichTransactionInfo, SignedUserTransaction, TransactionInfo};
use starcoin_types::vm_error::KeptVMStatus;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::block_metadata::LegacyBlockMetadata;
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::transaction::LegacyTransaction;
use std::path::Path;

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
    let tmpdir = starcoin_config::temp_dir();
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
    let tmpdir = starcoin_config::temp_dir();
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
    let transaction_info3 = RichTransactionInfo::decode_value(&value3).unwrap();
    assert_eq!(transaction_info3, transaction_info1);
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
    let tmpdir = starcoin_config::temp_dir();

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

fn generate_old_block_data(instance: StorageInstance) -> Result<(Vec<HashValue>, Vec<HashValue>)> {
    const BLOCK_COUNT: u64 = 1001;
    let old_block_header_storage = OldBlockHeaderStorage::new(instance.clone());
    let old_block_storage = OldBlockInnerStorage::new(instance.clone());
    let old_failed_block_storage = OldFailedBlockStorage::new(instance);

    let failed_block_ids = (0..BLOCK_COUNT)
        .map(|_| {
            let failed_block = FailedBlock::random();
            let failed_block_id = {
                let (block, _, _, _) = failed_block.clone().into();
                block.id()
            };
            let old_failed_block: OldFailedBlockV2 = failed_block.into();
            old_failed_block_storage
                .put(failed_block_id, old_failed_block)
                .unwrap();
            failed_block_id
        })
        .collect::<Vec<_>>();

    let block_ids = (0..BLOCK_COUNT)
        .map(|_| {
            let block = Block::random();
            let block_id = block.id();
            let old_block = block.clone().into();
            let old_block_header = block.header.into();

            old_block_storage.put(block_id, old_block).unwrap();
            old_block_header_storage
                .put(block_id, old_block_header)
                .unwrap();
            block_id
        })
        .collect::<Vec<_>>();

    Ok((block_ids, failed_block_ids))
}

fn generate_old_db(path: &Path) -> Result<(Vec<HashValue>, Vec<HashValue>, Vec<HashValue>)> {
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(path, RocksdbConfig::default(), None)?,
    );
    let storage = Storage::new(instance.clone())?;
    let old_transaction_info_storage = OldTransactionInfoStorage::new(instance.clone());
    let old_transaction_storage = LegacyTransactionStorage::new(instance.clone());

    let txn = SignedUserTransaction::mock();
    let body = BlockBody::new(vec![txn.clone()], None);
    let block_header = BlockHeader::rational_random(body.hash());
    let block = Block::new(block_header.clone(), body);
    let mut txn_inf_ids = vec![];
    let mut txn_ids = vec![];
    let block_metadata: LegacyBlockMetadata = block.to_metadata(0).into();
    let txn_info_0 = TransactionInfo::new(
        block_metadata.id(),
        HashValue::random(),
        vec![].as_slice(),
        0,
        KeptVMStatus::Executed,
    );
    let txn_0 = LegacyTransaction::BlockMetadata(block_metadata);
    txn_ids.push(txn_0.id());
    old_transaction_storage.save_transaction(txn_0)?;
    txn_inf_ids.push(txn_info_0.id());
    let txn_info_1 = TransactionInfo::new(
        txn.id(),
        HashValue::random(),
        vec![].as_slice(),
        100,
        KeptVMStatus::Executed,
    );
    let txn_1 = LegacyTransaction::UserTransaction(txn);
    txn_ids.push(txn_1.id());
    old_transaction_storage.save_transaction(txn_1)?;
    txn_inf_ids.push(txn_info_1.id());
    let block_info = BlockInfo::new(
        block_header.id(),
        0.into(),
        AccumulatorInfo::new(HashValue::random(), vec![], 2, 3),
        AccumulatorInfo::new(HashValue::random(), vec![], 1, 1),
    );
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

    let (block_ids, failed_block_ids) = generate_old_block_data(instance)?;

    Ok((txn_inf_ids, block_ids, failed_block_ids))
}

#[stest::test]
#[ignore]
pub fn test_db_upgrade() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let (txn_info_ids, block_ids, failed_block_ids) = generate_old_db(tmpdir.path())?;
    info!("Upgrade blocks:{},{:?}", block_ids.len(), block_ids);
    let mut instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None)?,
    );

    instance.check_upgrade()?;
    let storage = Storage::new(instance.clone())?;
    let old_block_header_storage = OldBlockHeaderStorage::new(instance.clone());
    let old_block_storage = OldBlockInnerStorage::new(instance.clone());
    let old_failed_block_storage = OldFailedBlockStorage::new(instance.clone());
    let old_transaction_info_storage = OldTransactionInfoStorage::new(instance);

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

    for block_id in block_ids {
        assert!(
            old_block_header_storage.get(block_id)?.is_none(),
            "expect OldBlockHeader is none"
        );
        assert!(
            storage.get_block_header_by_hash(block_id)?.is_some(),
            "expect BlockHeader is some"
        );

        assert!(
            old_block_storage.get(block_id)?.is_none(),
            "expect OldBlock is none"
        );
        assert!(
            storage.get_block_by_hash(block_id)?.is_some(),
            "expect Block is some"
        );
    }

    for failed_block_id in failed_block_ids {
        assert!(
            old_failed_block_storage.get(failed_block_id)?.is_none(),
            "expect OldBlock is none"
        );
        assert!(
            storage.get_failed_block_by_id(failed_block_id)?.is_some(),
            "expect Block is some"
        );
    }

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
    let id1 = transaction_info1.id();

    let transaction_info2 = RichTransactionInfo::new(
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
    let id2 = transaction_info2.id();

    let transaction_info3 = RichTransactionInfo::new(
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
        TRANSACTION_INFO_PREFIX_NAME_V2,
        vec![id1.to_vec(), id2.to_vec(), id3.to_vec()],
    )?;
    assert!(&cache_infos.first().unwrap().is_none(), "id1 has evicted");
    assert_eq!(
        RichTransactionInfo::decode_value(&cache_infos.get(1).unwrap().clone().unwrap())?,
        transaction_info2
    );
    assert_eq!(
        RichTransactionInfo::decode_value(&cache_infos.get(2).unwrap().clone().unwrap())?,
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
    let key1 = TableHandle(AccountAddress::random());
    let table_info1 = TableInfo::new(TypeTag::U8, TypeTag::U8);
    storage.save_table_info(key1, table_info1.clone())?;
    let val = storage.get_table_info(key1);
    assert!(val.is_ok());
    assert_eq!(val.unwrap().unwrap(), table_info1);
    let key2 = TableHandle(AccountAddress::random());
    let val = storage.get_table_info(key2);
    assert!(val.is_ok());
    assert_eq!(val.unwrap(), None);
    let keys = vec![
        TableHandle(AccountAddress::random()),
        TableHandle(AccountAddress::random()),
    ];
    let vals = vec![
        TableInfo::new(TypeTag::U8, TypeTag::Address),
        TableInfo::new(TypeTag::Address, TypeTag::U128),
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
        .collect::<Vec<TableInfo>>();
    assert_eq!(vals, vals2);
    Ok(())
}

fn run_write_batch(instance: StorageInstance) -> Result<()> {
    let body = BlockBody::new_empty();

    let block1 = Block::new(
        BlockHeaderBuilder::new()
            .with_body_hash(body.hash())
            .with_number(1)
            .build(),
        body.clone(),
    );
    let block2 = Block::new(
        BlockHeaderBuilder::new()
            .with_body_hash(body.hash())
            .with_number(2)
            .build(),
        body.clone(),
    );

    let dag_block1 = DagSyncBlock {
        block: Block::new(
            BlockHeaderBuilder::new()
                .with_body_hash(body.hash())
                .with_number(3)
                .build(),
            body.clone(),
        ),
        children: vec![Block::random().id(), Block::random().id()],
    };

    let dag_block2 = DagSyncBlock {
        block: Block::new(
            BlockHeaderBuilder::new()
                .with_body_hash(body.hash())
                .with_number(4)
                .build(),
            body.clone(),
        ),
        children: vec![Block::random().id(), Block::random().id()],
    };

    let batch_with_columns = WriteBatchWithColumn {
        data: vec![
            WriteBatchData {
                column: BLOCK_PREFIX_NAME.to_string(),
                row_data: WriteBatch::new_with_rows(vec![
                    (
                        block1.id().encode()?,
                        WriteOp::Value(block1.clone().encode()?),
                    ),
                    (
                        block2.id().encode()?,
                        WriteOp::Value(block2.clone().encode()?),
                    ),
                ]),
            },
            WriteBatchData {
                column: DAG_SYNC_BLOCK_PREFIX_NAME.to_string(),
                row_data: WriteBatch::new_with_rows(vec![
                    (
                        dag_block1.block.id().encode()?,
                        WriteOp::Value(dag_block1.clone().encode()?),
                    ),
                    (
                        dag_block2.block.id().encode()?,
                        WriteOp::Value(dag_block2.clone().encode()?),
                    ),
                ]),
            },
        ],
    };

    instance.write_batch_with_column(batch_with_columns)?;

    match instance {
        StorageInstance::CACHE { cache } => {
            let read_block1 = Block::decode(
                &cache
                    .get(BLOCK_PREFIX_NAME, block1.id().encode()?)?
                    .expect("failed to get the block"),
            )?;
            assert_eq!(read_block1, block1);

            let read_block2 = Block::decode(
                &cache
                    .get(BLOCK_PREFIX_NAME, block2.id().encode()?)?
                    .expect("failed to get the block"),
            )?;
            assert_eq!(read_block2, block2);

            let read_dag_block1 = DagSyncBlock::decode(
                &cache
                    .get(DAG_SYNC_BLOCK_PREFIX_NAME, dag_block1.block.id().encode()?)?
                    .expect("failed to get the dag block"),
            )?;
            assert_eq!(read_dag_block1, dag_block1);

            let read_dag_block2 = DagSyncBlock::decode(
                &cache
                    .get(DAG_SYNC_BLOCK_PREFIX_NAME, dag_block2.block.id().encode()?)?
                    .expect("failed to get the dag block"),
            )?;
            assert_eq!(read_dag_block2, dag_block2);
        }
        StorageInstance::DB { db } => {
            let read_block1 = Block::decode(
                &db.get(BLOCK_PREFIX_NAME, block1.id().encode()?)?
                    .expect("failed to get the block"),
            )?;
            assert_eq!(read_block1, block1);

            let read_block2 = Block::decode(
                &db.get(BLOCK_PREFIX_NAME, block2.id().encode()?)?
                    .expect("failed to get the block"),
            )?;
            assert_eq!(read_block2, block2);

            let read_dag_block1 = DagSyncBlock::decode(
                &db.get(DAG_SYNC_BLOCK_PREFIX_NAME, dag_block1.block.id().encode()?)?
                    .expect("failed to get the dag block"),
            )?;
            assert_eq!(read_dag_block1, dag_block1);

            let read_dag_block2 = DagSyncBlock::decode(
                &db.get(DAG_SYNC_BLOCK_PREFIX_NAME, dag_block2.block.id().encode()?)?
                    .expect("failed to get the dag block"),
            )?;
            assert_eq!(read_dag_block2, dag_block2);
        }
        StorageInstance::CacheAndDb { cache, db } => {
            let read_block1 = Block::decode(
                &cache
                    .get(BLOCK_PREFIX_NAME, block1.id().encode()?)?
                    .expect("failed to get the block"),
            )?;
            assert_eq!(read_block1, block1);

            let read_block2 = Block::decode(
                &cache
                    .get(BLOCK_PREFIX_NAME, block2.id().encode()?)?
                    .expect("failed to get the block"),
            )?;
            assert_eq!(read_block2, block2);

            let read_dag_block1 = DagSyncBlock::decode(
                &cache
                    .get(DAG_SYNC_BLOCK_PREFIX_NAME, dag_block1.block.id().encode()?)?
                    .expect("failed to get the dag block"),
            )?;
            assert_eq!(read_dag_block1, dag_block1);

            let read_dag_block2 = DagSyncBlock::decode(
                &cache
                    .get(DAG_SYNC_BLOCK_PREFIX_NAME, dag_block2.block.id().encode()?)?
                    .expect("failed to get the dag block"),
            )?;
            assert_eq!(read_dag_block2, dag_block2);

            let read_block1 = Block::decode(
                &db.get(BLOCK_PREFIX_NAME, block1.id().encode()?)?
                    .expect("failed to get the block"),
            )?;
            assert_eq!(read_block1, block1);

            let read_block2 = Block::decode(
                &db.get(BLOCK_PREFIX_NAME, block2.id().encode()?)?
                    .expect("failed to get the block"),
            )?;
            assert_eq!(read_block2, block2);

            let read_dag_block1 = DagSyncBlock::decode(
                &db.get(DAG_SYNC_BLOCK_PREFIX_NAME, dag_block1.block.id().encode()?)?
                    .expect("failed to get the dag block"),
            )?;
            assert_eq!(read_dag_block1, dag_block1);

            let read_dag_block2 = DagSyncBlock::decode(
                &db.get(DAG_SYNC_BLOCK_PREFIX_NAME, dag_block2.block.id().encode()?)?
                    .expect("failed to get the dag block"),
            )?;
            assert_eq!(read_dag_block2, dag_block2);
        }
    }

    Ok(())
}

#[test]
fn test_batch_write_for_cache_and_db() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None)?,
    );

    run_write_batch(instance)
}

#[test]
fn test_batch_write_for_db() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let instance = StorageInstance::new_db_instance(DBStorage::new(
        tmpdir.path(),
        RocksdbConfig::default(),
        None,
    )?);

    run_write_batch(instance)
}

#[test]
fn test_batch_write_for_cache() -> Result<()> {
    let instance = StorageInstance::new_cache_instance();

    run_write_batch(instance)
}
