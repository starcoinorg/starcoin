// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use chrono::prelude::*;
use crypto::HashValue;

use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::StorageInstance;
use crate::Storage;
use logger::prelude::*;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{Block, BlockBody, BlockHeader, BlockState};
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_types::U256;
use std::sync::Arc;

#[test]
fn test_block() {
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage,
        db_storage,
    ))
    .unwrap();
    let consensus_header = vec![0u8; 1];
    let dt = Local::now();

    let block_header1 = BlockHeader::new(
        HashValue::random(),
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        1,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        U256::zero(),
        consensus_header,
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
    let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()]);
    storage
        .block_storage
        .save_body(block_id, block_body1.clone())
        .unwrap();
    let block1 = Block::new(block_header1.clone(), block_body1);
    // save block1
    storage
        .block_storage
        .save(block1.clone(), BlockState::Executed)
        .unwrap();
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
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage,
        db_storage,
    ))
    .unwrap();
    let consensus_header = vec![0u8; 1];
    let dt = Local::now();

    let block_header1 = BlockHeader::new(
        HashValue::random(),
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        0,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        U256::zero(),
        consensus_header,
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
    let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()]);
    storage
        .block_storage
        .save_body(block_id, block_body1.clone())
        .unwrap();
    let block1 = Block::new(block_header1.clone(), block_body1);

    // save block1
    storage
        .block_storage
        .save(block1.clone(), BlockState::Executed)
        .unwrap();
    let block_number1 = block_header1.number();
    storage
        .block_storage
        .save_number(block_number1, block_id)
        .unwrap();
    //read to block2
    let block2 = storage.block_storage.get(block_id).unwrap();
    assert!(block2.is_some());
    assert_eq!(block1, block2.unwrap());
    //get number to block3
    let block3 = storage
        .block_storage
        .get_block_by_number(block_number1)
        .unwrap()
        .unwrap();
    assert_eq!(block1, block3);
    //get header by number
    let block4_header = storage
        .block_storage
        .get_block_header_by_number(block_number1)
        .unwrap()
        .unwrap();
    assert_eq!(block_header1, block4_header);
}

#[test]
fn test_branch_number() {
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage,
        db_storage,
    ))
    .unwrap();
    let consensus_header = vec![0u8; 1];
    let dt = Local::now();

    let block_header1 = BlockHeader::new(
        HashValue::random(),
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        1,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        U256::zero(),
        consensus_header,
    );
    storage
        .block_storage
        .save_header(block_header1.clone())
        .unwrap();
    let block_id = block_header1.id();
    let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()]);
    storage
        .block_storage
        .save_body(block_id, block_body1.clone())
        .unwrap();
    let block1 = Block::new(block_header1.clone(), block_body1);

    // save block1
    storage
        .block_storage
        .save(block1.clone(), BlockState::Executed)
        .unwrap();
    let block_number1 = block_header1.number();
    let branch_id = HashValue::random();
    storage
        .block_storage
        .save_branch_number(branch_id, block_number1, block_id)
        .unwrap();
    //read to branch number
    let block_id2 = storage
        .block_storage
        .get_branch_number(branch_id, block_number1)
        .unwrap()
        .unwrap();
    assert_eq!(block_id2, block_id);

    //get branch number to block3
    let block3 = storage
        .block_storage
        .get_block_by_branch_number(branch_id, block_number1)
        .unwrap()
        .unwrap();
    assert_eq!(block1, block3);
    //get header by branch number
    let block4_header = storage
        .block_storage
        .get_header_by_branch_number(branch_id, block_number1)
        .unwrap()
        .unwrap();
    assert_eq!(block_header1, block4_header);
}

#[test]
fn test_block_branch_hashes() {
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage,
        db_storage,
    ))
    .unwrap();
    let consensus_header = vec![0u8; 1];
    let dt = Local::now();

    let block_header0 = BlockHeader::new(
        HashValue::random(),
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        0,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        U256::zero(),
        consensus_header.clone(),
    );
    storage
        .block_storage
        .save_header(block_header0.clone())
        .unwrap();

    let parent_hash = block_header0.id();
    let block_header1 = BlockHeader::new(
        parent_hash,
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        1,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        U256::zero(),
        consensus_header.clone(),
    );
    storage
        .block_storage
        .save_header(block_header1.clone())
        .unwrap();
    let block_id = block_header1.id();
    debug!("header1: {}", block_id.to_hex());
    let block_header2 = BlockHeader::new(
        parent_hash,
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        2,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        U256::zero(),
        consensus_header.clone(),
    );
    storage
        .block_storage
        .save_header(block_header2.clone())
        .unwrap();
    debug!("header2: {}", block_header2.id().to_hex());

    let block_header3 = BlockHeader::new(
        block_id,
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        3,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        U256::zero(),
        consensus_header.clone(),
    );
    storage
        .block_storage
        .save_header(block_header3.clone())
        .unwrap();
    debug!("header3: {}", block_header3.id().to_hex());

    let block_header4 = BlockHeader::new(
        block_header3.id(),
        HashValue::random(),
        dt.timestamp_nanos() as u64,
        4,
        AccountAddress::random(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        0,
        U256::zero(),
        consensus_header,
    );
    storage
        .block_storage
        .save_header(block_header4.clone())
        .unwrap();
    debug!("header4: {}", block_header4.id().to_hex());
    let hashes = storage
        .block_storage
        .get_branch_hashes(block_header4.id())
        .unwrap();
    let desert_vec = vec![block_header3.id(), block_id];
    assert_eq!(hashes, desert_vec);
    let comm_hash = storage
        .block_storage
        .get_common_ancestor(block_header1.id(), block_header2.id())
        .unwrap()
        .unwrap();
    assert_eq!(comm_hash, parent_hash);
}
