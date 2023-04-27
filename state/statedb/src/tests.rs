use super::*;
use starcoin_state_tree::mock::MockStateNodeStore;
use starcoin_types::access_path::AccessPath;
use starcoin_types::write_set::{WriteOp, WriteSet, WriteSetMut};
use starcoin_vm_types::account_config::AccountResource;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::state_store::state_key::{StateKey, TableItem};
use std::collections::HashMap;

fn random_bytes() -> Vec<u8> {
    HashValue::random().to_vec()
}

fn to_write_set(access_path: AccessPath, value: Vec<u8>) -> WriteSet {
    WriteSetMut::new(vec![(
        StateKey::AccessPath(access_path),
        WriteOp::Value(value),
    )])
    .freeze()
    .expect("freeze write_set must success.")
}

fn state_keys_to_write_set(state_keys: Vec<StateKey>, values: Vec<Vec<u8>>) -> WriteSet {
    WriteSetMut::new(
        state_keys
            .into_iter()
            .zip(values)
            .into_iter()
            .map(|(key, val)| (key, WriteOp::Value(val)))
            .collect::<Vec<_>>(),
    )
    .freeze()
    .expect("freeze write_set must success.")
}

#[test]
fn test_state_proof() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let access_path = AccessPath::random_resource();
    let state0 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(access_path.clone(), state0.clone()))?;

    let state_root = chain_state_db.commit()?;
    let state1 = chain_state_db.get_state_value(&StateKey::AccessPath(access_path.clone()))?;
    assert!(state1.is_some());
    assert_eq!(state0, state1.unwrap());
    println!("{}", access_path.address.key_hash());
    println!("{}", access_path.key_hash());
    let state_with_proof = chain_state_db.get_with_proof(&access_path)?;
    println!("{:?}", state_with_proof);
    state_with_proof
        .proof
        .verify(state_root, access_path, state_with_proof.state.as_deref())?;
    Ok(())
}

#[test]
fn test_state_db() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let access_path = AccessPath::random_resource();

    let state0 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(access_path.clone(), state0))?;
    let state_root = chain_state_db.commit()?;

    let state1 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(access_path, state1))?;
    let new_state_root = chain_state_db.commit()?;
    assert_ne!(state_root, new_state_root);
    Ok(())
}

#[test]
fn test_state_db_dump_and_apply() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let access_path = AccessPath::random_resource();
    let state0 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(access_path, state0))?;
    chain_state_db.commit()?;
    chain_state_db.flush()?;

    let global_state = chain_state_db.dump()?;
    assert_eq!(
        global_state.state_sets().len(),
        1,
        "unexpect state_set length."
    );

    let storage2 = MockStateNodeStore::new();
    let chain_state_db2 = ChainStateDB::new(Arc::new(storage2), None);
    chain_state_db2.apply(global_state.clone())?;
    let global_state2 = chain_state_db2.dump()?;
    assert_eq!(global_state2.state_sets().len(), 1);
    assert_eq!(global_state, global_state2);

    Ok(())
}

#[test]
fn test_state_version() -> Result<()> {
    let storage = Arc::new(MockStateNodeStore::new());
    let chain_state_db = ChainStateDB::new(storage.clone(), None);
    let account_address = AccountAddress::random();
    let access_path = AccessPath::new(account_address, AccountResource::resource_path());
    let old_state = random_bytes();
    chain_state_db.apply_write_set(to_write_set(access_path.clone(), old_state.clone()))?;
    chain_state_db.commit()?;
    chain_state_db.flush()?;
    let old_root = chain_state_db.state_root();

    let new_state = random_bytes();
    chain_state_db.apply_write_set(to_write_set(access_path.clone(), new_state))?;

    let chain_state_db_ori = ChainStateDB::new(storage, Some(old_root));
    let old_state2 = chain_state_db_ori
        .get_state_value(&StateKey::AccessPath(access_path))?
        .unwrap();
    assert_eq!(old_state, old_state2);

    Ok(())
}

#[test]
fn test_state_db_dump_iter() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let access_path1 = AccessPath::random_resource();
    let state1 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(access_path1, state1))?;
    let access_path2 = AccessPath::random_resource();
    let state2 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(access_path2, state2))?;
    chain_state_db.commit()?;
    chain_state_db.flush()?;

    let global_state1 = chain_state_db.dump()?;
    assert_eq!(
        global_state1.state_sets().len(),
        2,
        "unexpected state_set length."
    );
    let mut kv1 = HashMap::new();
    for item in global_state1.into_inner() {
        kv1.insert(item.0, item.1);
    }
    let mut kv2 = HashMap::new();
    let global_states_iter = chain_state_db.dump_iter()?;
    for item in global_states_iter {
        kv2.insert(item.0, item.1);
    }
    assert_eq!(kv1, kv2);
    Ok(())
}

fn check_write_set(chain_state_db: &ChainStateDB, write_set: &WriteSet) -> Result<()> {
    for (state_key, value) in write_set.iter() {
        let val = chain_state_db.get_state_value(state_key)?;
        assert!(val.is_some());
        assert_eq!(WriteOp::Value(val.unwrap()), *value);
    }
    Ok(())
}

#[test]
fn test_state_db_with_table_item_once() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let handle1 = TableHandle(AccountAddress::random());
    let handle2 = TableHandle(AccountAddress::random());
    let key2 = random_bytes();
    let val2 = random_bytes();
    let key3 = random_bytes();
    let val3 = random_bytes();
    let key4 = random_bytes();
    let val4 = random_bytes();
    let key5 = random_bytes();
    let val5 = random_bytes();
    let state_keys = vec![
        StateKey::AccessPath(AccessPath::random_code()),
        StateKey::AccessPath(AccessPath::random_resource()),
        StateKey::TableItem(TableItem {
            handle: handle1,
            key: key2.clone(),
        }),
        StateKey::TableItem(TableItem {
            handle: handle1,
            key: key3.clone(),
        }),
        StateKey::TableItem(TableItem {
            handle: handle2,
            key: key4.clone(),
        }),
        StateKey::TableItem(TableItem {
            handle: handle2,
            key: key5.clone(),
        }),
    ];
    let values = vec![
        random_bytes(),
        random_bytes(),
        val2.clone(),
        val3.clone(),
        val4.clone(),
        val5.clone(),
    ];
    let write_set = state_keys_to_write_set(state_keys, values);
    let write_set1 = write_set.clone();
    chain_state_db.apply_write_set(write_set)?;
    check_write_set(&chain_state_db, &write_set1)?;
    chain_state_db.commit()?;
    check_write_set(&chain_state_db, &write_set1)?;
    chain_state_db.flush()?;
    check_write_set(&chain_state_db, &write_set1)?;

    let storage1 = MockStateNodeStore::new();
    let storage2 = MockStateNodeStore::new();
    let table_handle_state1 =
        TableHandleStateObject::new(handle1, Arc::new(storage1), *SPARSE_MERKLE_PLACEHOLDER_HASH);
    let table_handle_state2 =
        TableHandleStateObject::new(handle2, Arc::new(storage2), *SPARSE_MERKLE_PLACEHOLDER_HASH);

    table_handle_state1.set(key2, val2);
    table_handle_state1.set(key3, val3);
    table_handle_state2.set(key4, val4);
    table_handle_state2.set(key5, val5);
    table_handle_state1.commit()?;
    table_handle_state1.flush()?;
    table_handle_state2.commit()?;
    table_handle_state2.flush()?;

    let storage3 = MockStateNodeStore::new();
    let state_tree_table_handles = StateTree::new(Arc::new(storage3), None);
    state_tree_table_handles.put(handle1, table_handle_state1.root_hash().to_vec());
    state_tree_table_handles.put(handle2, table_handle_state2.root_hash().to_vec());
    state_tree_table_handles.commit()?;
    state_tree_table_handles.flush()?;

    // XXX FIXME YSG
    assert_eq!(
        chain_state_db.table_handles_root_hash(0),
        state_tree_table_handles.root_hash()
    );

    // XXX FIXME YSG
    assert_eq!(
        chain_state_db.table_handle_address_root_hash(0),
        state_tree_table_handles.root_hash()
    );
    Ok(())
}

#[test]
fn test_state_with_table_item_proof() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let handle1 = TableHandle(AccountAddress::random());
    let handle2 = TableHandle(AccountAddress::random());
    let key1 = random_bytes();
    let val1 = random_bytes();
    let key2 = random_bytes();
    let val2 = random_bytes();
    let key3 = random_bytes();
    let val3 = random_bytes();
    let state_keys = vec![
        StateKey::AccessPath(AccessPath::random_code()),
        StateKey::AccessPath(AccessPath::random_resource()),
        StateKey::TableItem(TableItem {
            handle: handle1,
            key: key1.clone(),
        }),
        StateKey::TableItem(TableItem {
            handle: handle1,
            key: key2.clone(),
        }),
        StateKey::TableItem(TableItem {
            handle: handle2,
            key: key3.clone(),
        }),
    ];
    let values = vec![random_bytes(), random_bytes(), val1, val2, val3];
    let write_set = state_keys_to_write_set(state_keys, values);
    chain_state_db.apply_write_set(write_set)?;
    chain_state_db.commit()?;
    chain_state_db.flush()?;

    let state_with_table_item_proof1 =
        chain_state_db.get_with_table_item_proof(&handle1, key1.as_slice())?;
    state_with_table_item_proof1.verify(&handle1, key1.as_slice())?;
    let state_with_table_item_proof2 =
        chain_state_db.get_with_table_item_proof(&handle1, key2.as_slice())?;
    state_with_table_item_proof2.verify(&handle1, key2.as_slice())?;
    let state_with_table_item_proof3 =
        chain_state_db.get_with_table_item_proof(&handle2, key3.as_slice())?;
    state_with_table_item_proof3.verify(&handle2, key3.as_slice())?;
    Ok(())
}
