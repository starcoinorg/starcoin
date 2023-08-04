// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// use crate::DiemDB;
extern crate chrono;

use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::StorageInstance;
use crate::{Storage, WriteSetStore};
use starcoin_config::RocksdbConfig;
use starcoin_crypto::HashValue;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::state_store::state_key::{StateKey};
use starcoin_vm_types::state_store::table::TableHandle;
use starcoin_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};

fn to_write_set(access_path: AccessPath, value: Vec<u8>) -> WriteSet {
    WriteSetMut::new(vec![
        (
            StateKey::AccessPath(access_path.clone()),
            WriteOp::Value(value),
        ),
        (StateKey::AccessPath(access_path), WriteOp::Deletion),
    ])
        .freeze()
        .expect("freeze write_set must success.")
}

fn to_table_item_write_set(table_item: &StateKey, value: Vec<u8>) -> WriteSet {
    WriteSetMut::new(vec![
        (
            table_item.clone(),
            WriteOp::Value(value),
        ),
        (
            table_item.clone(),
            WriteOp::Deletion
        ),
    ]).freeze().expect("freeze write_set must success.")
}


#[test]
fn test_put_and_save() {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
        .unwrap();

    let access_path = AccessPath::random_resource();
    let state0 = HashValue::random().to_vec();
    let write_set = to_write_set(access_path.clone(), state0.clone());
    let hash = HashValue::random();

    //  Check save
    storage
        .write_set_store
        .save_write_set(hash, write_set)
        .expect("Save write set failed");
    let after = storage
        .write_set_store
        .get_write_set(hash)
        .expect("{} Write set not exists!")
        .expect("{} Write set not exists!");
    assert!(!after.is_empty());
    let mut iter = after.into_iter();
    let (st_key, op) = iter.next().expect("Error");
    assert_eq!(st_key, StateKey::AccessPath(access_path.clone()));
    assert_eq!(op, WriteOp::Value(state0));

    let (st_key, op) = iter.next().expect("Error");
    assert_eq!(st_key, StateKey::AccessPath(access_path));
    assert_eq!(op, WriteOp::Deletion);


}
#[test]
fn test_put_and_save_batch() {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    )).unwrap();

    let access_path = AccessPath::random_resource();
    let state0 = HashValue::random().to_vec();
    let write_set = to_write_set(access_path.clone(), state0.clone());
    let hash = HashValue::random();

    let data_batch = vec![(hash, write_set)];

    //  Check save
    storage.write_set_store
        .save_write_set_batch(data_batch)
        .expect("Save write set failed");
    let after = storage.write_set_store
        .get_write_set(hash)
        .expect("{} Write set not exists!")
        .expect("{} Write set not exists!");
    assert!(!after.is_empty());
    let mut iter = after.into_iter();
    let (st_key, op) = iter.next().expect("Error");
    assert_eq!(st_key, StateKey::AccessPath(access_path.clone()));
    assert_eq!(op, WriteOp::Value(state0));
    let (st_key, op) = iter.next().expect("Error");
    assert_eq!(st_key, StateKey::AccessPath(access_path));
    assert_eq!(op, WriteOp::Deletion);
}

#[test]
fn test_put_and_save_table_item() {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
        .unwrap();

    let table_item = StateKey::table_item(TableHandle(AccountAddress::random()), HashValue::random().to_vec());
    let table_item_val = HashValue::random().to_vec();
    let hash = HashValue::random();

    storage
        .write_set_store
        .save_write_set(hash, to_table_item_write_set(&table_item, table_item_val.clone()))
        .expect("Save write set failed");

    let after = storage
        .write_set_store
        .get_write_set(hash)
        .expect("{} Write set not exists!")
        .expect("{} Write set not exists!");

    assert!(!after.is_empty());

    let mut iter = after.into_iter();

    let (st_key, op) = iter.next().expect("Error");
    assert_eq!(st_key, table_item);
    assert_eq!(op, WriteOp::Value(table_item_val));

    let (st_key, op) = iter.next().expect("Error");
    assert_eq!(st_key, table_item);
    assert_eq!(op, WriteOp::Deletion);
}

#[test]
fn test_put_and_save_table_item_batch() {
    let tmpdir = starcoin_config::temp_dir();
    let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        DBStorage::new(tmpdir.path(), RocksdbConfig::default(), None).unwrap(),
    ))
        .unwrap();

    let table_item = StateKey::table_item(TableHandle(AccountAddress::random()), HashValue::random().to_vec());
    let table_item_val = HashValue::random().to_vec();
    let hash = HashValue::random();

    let batch_data = vec![(hash, to_table_item_write_set(&table_item, table_item_val.clone()))];
    storage
        .write_set_store
        .save_write_set_batch(batch_data)
        .expect("Save write set failed");

    let after = storage
        .write_set_store
        .get_write_set(hash)
        .expect("{} Write set not exists!")
        .expect("{} Write set not exists!");

    assert!(!after.is_empty());

    let mut iter = after.into_iter();

    let (st_key, op) = iter.next().expect("Error");
    assert_eq!(st_key, table_item);
    assert_eq!(op, WriteOp::Value(table_item_val));

    let (st_key, op) = iter.next().expect("Error");
    assert_eq!(st_key, table_item);
    assert_eq!(op, WriteOp::Deletion);
}
