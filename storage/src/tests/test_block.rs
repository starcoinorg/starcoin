// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use std::vec;

use anyhow::Ok;
use bcs_ext::BCSCodec;
use chrono::prelude::*;
use starcoin_crypto::HashValue;

use crate::block::{DagSyncBlock, FailedBlock, OldFailedBlock};
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::StorageInstance;
use crate::{BlockStore, Storage};
use starcoin_config::RocksdbConfig;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{Block, BlockBody, BlockHeader, BlockHeaderExtra};
use starcoin_types::genesis_config::ChainId;
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_uint::U256;

#[test]
fn test_block() {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
    .unwrap();
    let dt = Local::now();

    let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()], None);
    let block_header1 = BlockHeader::new(
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        1,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::random(),
        HashValue::zero(),
        0,
        U256::zero(),
        block_body1.hash(),
        ChainId::test(),
        0,
        BlockHeaderExtra::new([0u8; 4]),
        vec![],
        rand::random(),
        HashValue::random(),
    );
    storage
        .block_storage
        .save_header(block_header1.clone())
        .unwrap();
    let block_id = block_header1.id();
    assert_eq!(
        block_header1,
        storage
            .block_storage
            .get_block_header_by_hash(block_id)
            .unwrap()
            .unwrap()
    );
    storage
        .block_storage
        .save_body(block_id, block_body1.clone())
        .unwrap();
    let block1 = Block::new(block_header1.clone(), block_body1);
    // save block1
    storage.block_storage.save(block1.clone()).unwrap();
    //read to block2
    let block2 = storage.block_storage.get(block_id).unwrap();
    assert!(block2.is_some());
    assert_eq!(block1, block2.unwrap());
    //get header to block3
    let block_header3 = storage
        .block_storage
        .get_block_header_by_hash(block_id)
        .unwrap()
        .unwrap();
    assert_eq!(block_header1, block_header3);
}

#[test]
fn test_block_number() {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
    .unwrap();
    let dt = Local::now();

    let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()], None);
    let block_header1 = BlockHeader::new(
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        rand::random(),
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::random(),
        HashValue::zero(),
        0,
        U256::zero(),
        block_body1.hash(),
        ChainId::test(),
        0,
        BlockHeaderExtra::new([0u8; 4]),
        vec![],
        rand::random(),
        HashValue::random(),
    );
    storage
        .block_storage
        .save_header(block_header1.clone())
        .unwrap();
    let block_id = block_header1.id();
    assert_eq!(
        storage
            .block_storage
            .get_block_header_by_hash(block_id)
            .unwrap()
            .unwrap(),
        block_header1
    );
    storage
        .block_storage
        .save_body(block_id, block_body1.clone())
        .unwrap();
    let block1 = Block::new(block_header1, block_body1);

    // save block1
    storage.block_storage.save(block1.clone()).unwrap();

    //read to block2
    let block2 = storage.block_storage.get(block_id).unwrap();
    assert!(block2.is_some());
    assert_eq!(block1, block2.unwrap());
}

#[test]
fn test_old_failed_block_decode() {
    let dt = Local::now();
    let block_body = BlockBody::new(vec![SignedUserTransaction::mock()], None);
    let block_header = BlockHeader::new(
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        2,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::random(),
        HashValue::zero(),
        0,
        U256::zero(),
        block_body.hash(),
        ChainId::test(),
        0,
        BlockHeaderExtra::new([0u8; 4]),
        vec![],
        rand::random(),
        HashValue::random(),
    );

    let block = Block::new(block_header, block_body);

    let old_failed_block: OldFailedBlock = (block, None, "test decode".to_string()).into();
    let encoded = old_failed_block.encode();
    assert!(encoded.is_ok());
    let result = FailedBlock::decode(encoded.unwrap().as_slice());
    assert!(result.is_err());
}

#[test]
fn test_save_failed_block() {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
    .unwrap();
    let dt = Local::now();

    let block_body = BlockBody::new(vec![SignedUserTransaction::mock()], None);

    let block_header = BlockHeader::new(
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        3,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::random(),
        HashValue::zero(),
        0,
        U256::zero(),
        block_body.hash(),
        ChainId::test(),
        0,
        BlockHeaderExtra::new([0u8; 4]),
        vec![],
        rand::random(),
        HashValue::random(),
    );

    let block = Block::new(block_header, block_body);

    storage
        .block_storage
        .save_old_failed_block(
            block.id(),
            block.clone(),
            None,
            "test old block".to_string(),
        )
        .unwrap();

    let result = storage
        .block_storage
        .get_failed_block_by_id(block.id())
        .unwrap()
        .unwrap();
    assert_eq!(result.0, block);
    assert_eq!(result.3, "".to_string());

    storage
        .block_storage
        .save_failed_block(
            block.id(),
            block.clone(),
            None,
            "test old block".to_string(),
            "1".to_string(),
        )
        .unwrap();

    let result = storage
        .block_storage
        .get_failed_block_by_id(block.id())
        .unwrap()
        .unwrap();
    assert_eq!(result.0, block);
    assert_eq!(result.3, "1".to_string());
}

fn new_dag_sync_block(children: Vec<HashValue>) -> anyhow::Result<DagSyncBlock> {
    let dt = Local::now();
    let block_body = BlockBody::new(vec![SignedUserTransaction::mock()], None);
    let block_header = BlockHeader::new(
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        3,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::random(),
        HashValue::zero(),
        0,
        U256::zero(),
        block_body.hash(),
        ChainId::test(),
        0,
        BlockHeaderExtra::new([0u8; 4]),
        vec![],
        rand::random(),
        HashValue::random(),
    );

    let block = Block::new(block_header, block_body);

    anyhow::Ok(DagSyncBlock { block, children })
}

fn delete_disc_storage() -> anyhow::Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_db_instance(
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))?;

    let mut last_block_id = vec![];
    for _i in 0..10 {
        let block = new_dag_sync_block(last_block_id)?;
        storage.save_dag_sync_block(block.clone())?;
        last_block_id = vec![block.block.id()];
    }

    storage.delete_all_dag_sync_blocks()
}

fn delete_cache_storage() -> anyhow::Result<()> {
    let storage = Storage::new(StorageInstance::new_cache_instance())?;

    let mut last_block_id = vec![];
    for _i in 0..10 {
        let block = new_dag_sync_block(last_block_id)?;
        storage.save_dag_sync_block(block.clone())?;
        last_block_id = vec![block.block.id()];
    }

    storage.delete_all_dag_sync_blocks()
}

fn delete_disc_and_cache_storage() -> anyhow::Result<()> {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))?;

    let mut last_block_id = vec![];
    for _i in 0..10 {
        let block = new_dag_sync_block(last_block_id)?;
        storage.save_dag_sync_block(block.clone())?;
        last_block_id = vec![block.block.id()];
    }

    storage.delete_all_dag_sync_blocks()
}

#[test]
fn test_delete_sync_blocks() -> anyhow::Result<()> {
    delete_disc_storage()?;
    delete_cache_storage()?;
    delete_disc_and_cache_storage()?;

    Ok(())
}
