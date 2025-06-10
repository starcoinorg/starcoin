// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use crate::block::legacy::{BlockInnerStorage, FailedBlockStorage, OldFailedBlock};
use crate::block::{BlockHeaderStorage, StcBlockInnerStorage, StcFailedBlockStorage};
use crate::block_info::legacy::BlockInfoStorage;
use crate::block_info::StcBlockInfoStorage;
use crate::cache_storage::CacheStorage;
use crate::contract_event::legacy::ContractEventStorage;
use crate::db_storage::DBStorage;
use crate::storage::{
    CodecKVStore, InnerStore, KVStore, KeyCodec, SchemaStorage, StorageInstance, ValueCodec,
};
use crate::table_info::legacy::TableInfoStorage;
use crate::table_info::TableInfoStore;
use crate::tests::{random_txn_info, random_txn_info2};
use crate::transaction::legacy::TransactionStorage;
use crate::transaction_info::legacy::{BlockTransactionInfo, OldTransactionInfoStorage};
use crate::{
    BlockStore, BlockTransactionInfoStore, ContractEventStore, Storage, StorageVersion,
    TransactionStore, DEFAULT_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME,
    TRANSACTION_INFO_PREFIX_NAME_V3,
};
use anyhow::Result;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_config::RocksdbConfig;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_types::block::{Block, BlockHeaderExtra, BlockNumber};
use starcoin_types::transaction::StcRichTransactionInfo;
use starcoin_types::{
    account_address::AccountAddress,
    block::{legacy, BlockHeader},
    language_storage::TypeTag,
    startup_info::SnapshotRange,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
    vm_error::KeptVMStatus,
};
use starcoin_uint::U256;
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::event::EventKey;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
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

fn random_header_with_number(number: BlockNumber) -> BlockHeader {
    BlockHeader::new(
        HashValue::random(),
        rand::random(),
        number,
        AccountAddress::random(),
        HashValue::random(),
        HashValue::random(),
        HashValue::random(),
        rand::random(),
        U256::MAX,
        HashValue::random(),
        ChainId::test(),
        0,
        BlockHeaderExtra::new([0u8; 4]),
    )
}

fn generate_old_db(
    path: &Path,
) -> Result<(
    Vec<HashValue>,
    Vec<HashValue>,
    Vec<HashValue>,
    Vec<TableHandle>,
    Vec<HashValue>,
    Vec<HashValue>,
)> {
    let instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(path, RocksdbConfig::default(), None)?,
    );
    let old_transaction_info_storage = OldTransactionInfoStorage::new(instance.clone());
    let transaction_storage = TransactionStorage::new(instance.clone());
    let block_header_storage = BlockHeaderStorage::new(instance.clone());
    let block_storage = BlockInnerStorage::new(instance.clone());
    let block_info_storage = BlockInfoStorage::new(instance.clone());

    let block_header = random_header_with_number(1);
    let txn = SignedUserTransaction::mock();
    let block = legacy::Block {
        header: block_header.clone(),
        body: legacy::BlockBody {
            transactions: vec![txn.clone()],
            uncles: None,
        },
    };
    let mut txn_inf_ids = vec![];
    let mut txn_ids = vec![];
    let block_metadata = {
        // convert to latest type to construct block-meta txn just for convenient.
        let b = Block::from(block.clone());
        b.to_metadata(0)
    };
    let txn_info_0 = TransactionInfo::new(
        block_metadata.id(),
        HashValue::random(),
        vec![].as_slice(),
        0,
        KeptVMStatus::Executed,
    );

    let txn_meta = Transaction::BlockMetadata(block_metadata);
    txn_ids.push(txn_meta.id());
    transaction_storage.put(txn_meta.id(), txn_meta)?;
    txn_inf_ids.push(txn_info_0.id());
    let txn_info_1 = TransactionInfo::new(
        txn.id(),
        HashValue::random(),
        vec![].as_slice(),
        100,
        KeptVMStatus::Executed,
    );
    txn_inf_ids.push(txn_info_1.id());
    let block_info = legacy::BlockInfo {
        block_id: block_header.id(),
        total_difficulty: 0.into(),
        txn_accumulator_info: AccumulatorInfo::new(HashValue::random(), vec![], 2, 3),
        block_accumulator_info: AccumulatorInfo::new(HashValue::random(), vec![], 1, 1),
    };
    let user_txn = Transaction::UserTransaction(txn);
    txn_ids.push(user_txn.id());
    transaction_storage.put(user_txn.id(), user_txn)?;
    //commit_block(block)?;
    block_header_storage.put(block_header.id(), block_header.clone())?;
    block_storage.put(block_header.id(), block.clone())?;
    //save_block_info(block_info)?;
    block_info_storage.put(block_info.block_id, block_info.clone())?;

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

    let event_ids = generate_old_db_with_contract_event(instance.clone())?;
    let table_handles = generate_old_db_with_table_info(instance.clone())?;
    let (mut blocks, failed_blocks) = generate_old_block_and_block_info(instance)?;
    blocks.push(block_header.id());

    Ok((
        txn_inf_ids,
        txn_ids,
        event_ids,
        table_handles,
        blocks,
        failed_blocks,
    ))
}

fn generate_old_block_and_block_info(
    instance: StorageInstance,
) -> Result<(Vec<HashValue>, Vec<HashValue>)> {
    let block_header_storage = BlockHeaderStorage::new(instance.clone());
    let block_storage = BlockInnerStorage::new(instance.clone());
    let block_info_storage = BlockInfoStorage::new(instance.clone());
    let failed_block_storage = FailedBlockStorage::new(instance.clone());

    let num_blocks = 5u64;
    let num_failed_blocks = 2u64;
    let mut blocks = Vec::with_capacity(num_blocks as usize);
    let mut failed_blocks = Vec::with_capacity(num_failed_blocks as usize);
    for i in 0..num_blocks {
        let block_header = random_header_with_number(2 + i);
        let block_id = block_header.id();
        let block = legacy::Block {
            header: block_header.clone(),
            body: legacy::BlockBody {
                transactions: vec![],
                uncles: None,
            },
        };
        let block_info = legacy::BlockInfo {
            block_id: block_header.id(),
            total_difficulty: 0.into(),
            txn_accumulator_info: AccumulatorInfo::new(HashValue::random(), vec![], 2, 3),
            block_accumulator_info: AccumulatorInfo::new(HashValue::random(), vec![], 1, 1),
        };

        blocks.push(block_id);
        // commit_block
        block_header_storage.put(block_id, block_header)?;
        block_storage.put(block_id, block)?;
        // save block_info
        block_info_storage.put(block_id, block_info)?;
    }

    for i in 0..num_failed_blocks {
        let block_header = random_header_with_number(num_blocks + 2 + i);
        let block_id = block_header.id();
        let block = legacy::Block {
            header: block_header.clone(),
            body: legacy::BlockBody {
                transactions: vec![],
                uncles: None,
            },
        };
        if i == 0 {
            let old_failed_block = OldFailedBlock {
                block,
                peer_id: None,
                failed: format!("test old failed block {block_id}"),
            };
            failed_block_storage
                .get_store()
                .put(block_id.encode_key()?, old_failed_block.encode_value()?)?;
        } else {
            let failed_block = crate::block::legacy::FailedBlock {
                block,
                peer_id: None,
                failed: format!("test failed block{block_id}"),
                version: "v1".to_string(), // if not empty, it will be treated as a new version
            };
            failed_block_storage.put(block_id, failed_block)?;
        };
        info!("insert failed block: {}, idx {}", block_id, i);
        failed_blocks.push(block_id);
    }

    Ok((blocks, failed_blocks))
}

fn generate_old_db_with_contract_event(instance: StorageInstance) -> Result<Vec<HashValue>> {
    let contract_event_storage = ContractEventStorage::new(instance.clone());
    let mut contract_event_ids = vec![];

    for _ in 0..10 {
        let events = (0..10)
            .map(|i| ContractEvent::new(EventKey::random(), i as u64, TypeTag::Bool, vec![0u8; 32]))
            .collect::<Vec<_>>();
        // just use the event hash as the id for simplicity
        let key = events[0].crypto_hash();
        contract_event_storage.put(key, events)?;
        contract_event_ids.push(key);
    }
    Ok(contract_event_ids)
}

fn generate_old_db_with_table_info(instance: StorageInstance) -> Result<Vec<TableHandle>> {
    let table_info_storage = TableInfoStorage::new(instance.clone());
    let mut table_handles = vec![];

    for i in 0..12u8 {
        let table_handle = TableHandle(AccountAddress::new([i; AccountAddress::LENGTH]));
        let table_info = TableInfo::new(TypeTag::U8, TypeTag::U8);
        table_info_storage.put(table_handle, table_info)?;
        table_handles.push(table_handle);
    }
    Ok(table_handles)
}

#[stest::test]
fn test_db_upgrade() -> Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let (txn_info_ids, txn_ids, event_ids, table_handles, blocks, failed_blocks) =
        generate_old_db(tmpdir.path())?;
    let mut instance = StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None)?,
    );

    // set batch size to 5 to trigger multi-batches upgrade for ContractEventStorage and TableInfoStorage
    instance.check_upgrade(5)?;
    let storage = Storage::new(instance.clone())?;
    let old_transaction_info_storage = OldTransactionInfoStorage::new(instance.clone());

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
    let transaction_storage = TransactionStorage::new(instance.clone());
    for txn_id in txn_ids {
        assert!(
            transaction_storage.get(txn_id)?.is_none(),
            "expect Transaction is some"
        );
        assert!(
            storage
                .get_transaction(txn_id)?
                .and_then(|txn| txn.to_v1())
                .is_some(),
            "expect Transaction is some"
        );
    }

    let contract_event_storage = ContractEventStorage::new(instance.clone());
    for event_id in event_ids {
        assert!(
            contract_event_storage.get(event_id)?.is_none(),
            "expect ContractEvent is none"
        );

        assert_eq!(
            storage
                .get_contract_events_v2(event_id)?
                .map(|e| e.into_iter().all(|e| e.to_v1().is_some())),
            Some(true),
            "expect ContractEvents is none"
        );
    }

    let table_info_storage = TableInfoStorage::new(instance.clone());
    for table_handle in table_handles {
        assert!(
            table_info_storage.get(table_handle)?.is_none(),
            "expect TableInfo is none"
        );
        assert!(
            storage
                .get_table_info(table_handle.into())?
                .and_then(|ti| ti.to_v1())
                .is_some(),
            "expect TableInfo is none"
        );
    }

    let block_storage = BlockInnerStorage::new(instance.clone());
    let new_block_storage = StcBlockInnerStorage::new(instance.clone());
    let block_info_storage = BlockInfoStorage::new(instance.clone());
    let new_block_info_storage = StcBlockInfoStorage::new(instance.clone());

    for block_id in blocks {
        assert!(
            block_storage.get(block_id)?.is_none(),
            "expect Block is none"
        );
        assert!(
            new_block_storage.get(block_id)?.is_some(),
            "expect Block is some"
        );
        assert!(
            block_info_storage.get(block_id)?.is_none(),
            "expect BlockInfo is none"
        );
        assert!(
            new_block_info_storage.get(block_id)?.is_some(),
            "expect BlockInfo is some"
        );
    }

    let failed_block_storage = FailedBlockStorage::new(instance.clone());
    let new_failed_block_storage = StcFailedBlockStorage::new(instance.clone());

    for (idx, failed_block_id) in failed_blocks.into_iter().enumerate() {
        assert!(
            failed_block_storage.get(failed_block_id)?.is_none(),
            "expect FailedBlock is none"
        );
        let Some(failed_block) = new_failed_block_storage.get(failed_block_id)? else {
            panic!("expect FailedBlock is some");
        };
        let (_block, _peer_id, _failed, version) = failed_block.into();
        if idx == 0 {
            assert!(
                version.is_empty(),
                "expect old failed block version is empty"
            );
        } else {
            assert_eq!(
                version,
                "v1".to_string(),
                "expect new failed block version is v1"
            );
        }
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
