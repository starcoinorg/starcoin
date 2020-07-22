// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate chrono;

use chrono::prelude::*;
use crypto::HashValue;

use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::StorageInstance;
use crate::Storage;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{Block, BlockBody, BlockHeader, BlockState};
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_types::U256;
use std::sync::Arc;

#[test]
fn test_block() {
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = starcoin_config::temp_path();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage,
        db_storage,
    ))
    .unwrap();
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
        0,
        None,
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
    let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()], None);
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
    let tmpdir = starcoin_config::temp_path();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage,
        db_storage,
    ))
    .unwrap();
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
        0,
        None,
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
    let block_body1 = BlockBody::new(vec![SignedUserTransaction::mock()], None);
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
