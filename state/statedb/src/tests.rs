use super::*;
use starcoin_state_tree::mock::MockStateNodeStore;
use starcoin_types::access_path::AccessPath;
use starcoin_types::write_set::{WriteOp, WriteSet, WriteSetMut};
use starcoin_vm_types::access_path::random_identity;
use starcoin_vm_types::state_store::state_key::StateKey;
use std::collections::HashMap;

fn random_bytes() -> Vec<u8> {
    HashValue::random().to_vec()
}

fn to_write_set(state_key: StateKey, value: Vec<u8>) -> WriteSet {
    WriteSetMut::new(vec![(
        state_key,
        WriteOp::legacy_modification(value.into()),
    )])
    .freeze()
    .expect("freeze write_set must success.")
}

fn state_keys_to_write_set(state_keys: Vec<StateKey>, values: Vec<Vec<u8>>) -> WriteSet {
    WriteSetMut::new(
        state_keys
            .into_iter()
            .zip(values)
            .map(|(key, val)| (key, WriteOp::legacy_modification(val.into())))
            .collect::<Vec<_>>(),
    )
    .freeze()
    .expect("freeze write_set must success.")
}

#[test]
fn test_state_proof() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let account_address = AccountAddress::random();
    let struct_tag = StructTag {
        address: account_address,
        module: random_identity(),
        name: random_identity(),
        type_args: vec![],
    };
    let access_path = AccessPath::new(account_address, DataPath::Resource(struct_tag.clone()));
    let state_key = StateKey::resource(&account_address, &struct_tag);
    let state0 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(state_key.clone(), state0.clone()))?;

    let state_root = chain_state_db.commit()?;
    let state1 = chain_state_db.get_state_value_bytes(&state_key)?;
    assert!(state1.is_some());
    assert_eq!(state0, state1.unwrap().to_vec());
    println!("{}", account_address.key_hash());
    println!("{}", state_key.key_hash());
    let state_with_proof = chain_state_db.get_with_proof(&state_key)?;
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
    let account_address = AccountAddress::random();
    let struct_tag = StructTag {
        address: account_address,
        module: random_identity(),
        name: random_identity(),
        type_args: vec![],
    };
    let state_key = StateKey::resource(&account_address, &struct_tag);
    let state0 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(state_key.clone(), state0))?;
    let state_root = chain_state_db.commit()?;

    let state1 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(state_key, state1))?;
    let new_state_root = chain_state_db.commit()?;
    assert_ne!(state_root, new_state_root);
    Ok(())
}

#[test]
fn test_state_db_dump_and_apply() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let account_address = AccountAddress::random();
    let struct_tag = StructTag {
        address: account_address,
        module: random_identity(),
        name: random_identity(),
        type_args: vec![],
    };
    let state_key = StateKey::resource(&account_address, &struct_tag);

    let state0 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(state_key, state0))?;
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
    let struct_tag = StructTag {
        address: account_address,
        module: random_identity(),
        name: random_identity(),
        type_args: vec![],
    };
    let state_key = StateKey::resource(&account_address, &struct_tag);
    let old_state = random_bytes();
    chain_state_db.apply_write_set(to_write_set(state_key.clone(), old_state.clone()))?;
    chain_state_db.commit()?;
    chain_state_db.flush()?;
    let old_root = chain_state_db.state_root();

    let new_state = random_bytes();
    chain_state_db.apply_write_set(to_write_set(state_key.clone(), new_state))?;

    let chain_state_db_ori = ChainStateDB::new(storage, Some(old_root));
    let old_state2 = chain_state_db_ori
        .get_state_value_bytes(&state_key)?
        .unwrap();
    assert_eq!(old_state, old_state2.to_vec());

    Ok(())
}

#[test]
fn test_state_db_dump_iter() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let account_address = AccountAddress::random();
    let struct_tag = StructTag {
        address: account_address,
        module: random_identity(),
        name: random_identity(),
        type_args: vec![],
    };
    let state_key1 = StateKey::resource(&account_address, &struct_tag);
    let state1 = random_bytes();
    chain_state_db.apply_write_set(to_write_set(state_key1, state1))?;
    let state2 = random_bytes();
    let account_address2 = AccountAddress::random();
    let struct_tag2 = StructTag {
        address: account_address2,
        module: random_identity(),
        name: random_identity(),
        type_args: vec![],
    };
    let state_key2 = StateKey::resource(&account_address2, &struct_tag2);
    chain_state_db.apply_write_set(to_write_set(state_key2, state2))?;
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
        let val = chain_state_db.get_state_value_bytes(state_key)?;
        assert!(val.is_some());
        assert_eq!(WriteOp::legacy_modification(val.unwrap()), *value);
    }
    Ok(())
}

#[test]
fn test_state_db_with_table_item_once() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
    let handle11 = TableHandle(AccountAddress::from_hex_literal("0x20").unwrap());
    let handle12 = TableHandle(AccountAddress::from_hex_literal("0x40").unwrap());
    let handle21 = TableHandle(AccountAddress::from_hex_literal("0x21").unwrap());
    let handle22 = TableHandle(AccountAddress::from_hex_literal("0x41").unwrap());
    let handle31 = TableHandle(AccountAddress::from_hex_literal("0x22").unwrap());
    let handle32 = TableHandle(AccountAddress::from_hex_literal("0x42").unwrap());
    let handle41 = TableHandle(AccountAddress::from_hex_literal("0x23").unwrap());
    let handle42 = TableHandle(AccountAddress::from_hex_literal("0x43").unwrap());

    let key11 = random_bytes();
    let val11 = random_bytes();
    let key13 = random_bytes();
    let val13 = random_bytes();
    let key12 = random_bytes();
    let val12 = random_bytes();
    let key14 = random_bytes();
    let val14 = random_bytes();
    let key21 = random_bytes();
    let val21 = random_bytes();
    let key23 = random_bytes();
    let val23 = random_bytes();
    let key22 = random_bytes();
    let val22 = random_bytes();
    let key24 = random_bytes();
    let val24 = random_bytes();
    let key31 = random_bytes();
    let val31 = random_bytes();
    let key33 = random_bytes();
    let val33 = random_bytes();
    let key32 = random_bytes();
    let val32 = random_bytes();
    let key34 = random_bytes();
    let val34 = random_bytes();
    let key41 = random_bytes();
    let val41 = random_bytes();
    let key43 = random_bytes();
    let val43 = random_bytes();
    let key42 = random_bytes();
    let val42 = random_bytes();
    let key44 = random_bytes();
    let val44 = random_bytes();

    let account_address1 = AccountAddress::random();
    let module1 = random_identity();
    let state_key1 = StateKey::module(&account_address1, &module1);
    let account_address2 = AccountAddress::random();
    let struct_tag2 = StructTag {
        address: account_address2,
        module: random_identity(),
        name: random_identity(),
        type_args: vec![],
    };
    let state_key2 = StateKey::resource(&account_address2, &struct_tag2);

    let state_keys = vec![
        state_key1,
        state_key2,
        StateKey::table_item(&handle11, key11.as_slice()),
        StateKey::table_item(&handle11, key13.as_slice()),
        StateKey::table_item(&handle12, key12.as_slice()),
        StateKey::table_item(&handle12, key14.as_slice()),
        StateKey::table_item(&handle21, key21.as_slice()),
        StateKey::table_item(&handle21, key23.as_slice()),
        StateKey::table_item(&handle22, key22.as_slice()),
        StateKey::table_item(&handle22, key24.as_slice()),
        StateKey::table_item(&handle31, key31.as_slice()),
        StateKey::table_item(&handle31, key33.as_slice()),
        StateKey::table_item(&handle32, key32.as_slice()),
        StateKey::table_item(&handle32, key34.as_slice()),
        StateKey::table_item(&handle41, key41.as_slice()),
        StateKey::table_item(&handle41, key43.as_slice()),
        StateKey::table_item(&handle42, key42.as_slice()),
        StateKey::table_item(&handle42, key44.as_slice()),
    ];
    let values = vec![
        random_bytes(),
        random_bytes(),
        val11.clone(),
        val13.clone(),
        val12.clone(),
        val14.clone(),
        val21.clone(),
        val23.clone(),
        val22.clone(),
        val24.clone(),
        val31.clone(),
        val33.clone(),
        val32.clone(),
        val34.clone(),
        val41.clone(),
        val43.clone(),
        val42.clone(),
        val44.clone(),
    ];
    let write_set = state_keys_to_write_set(state_keys, values);
    let write_set1 = write_set.clone();
    chain_state_db.apply_write_set(write_set)?;
    check_write_set(&chain_state_db, &write_set1)?;
    chain_state_db.commit()?;
    check_write_set(&chain_state_db, &write_set1)?;
    chain_state_db.flush()?;
    check_write_set(&chain_state_db, &write_set1)?;

    let storage11 = MockStateNodeStore::new();
    let storage12 = MockStateNodeStore::new();
    let storage21 = MockStateNodeStore::new();
    let storage22 = MockStateNodeStore::new();
    let storage31 = MockStateNodeStore::new();
    let storage32 = MockStateNodeStore::new();
    let storage41 = MockStateNodeStore::new();
    let storage42 = MockStateNodeStore::new();

    let storage1 = MockStateNodeStore::new();
    let storage2 = MockStateNodeStore::new();
    let storage3 = MockStateNodeStore::new();
    let storage4 = MockStateNodeStore::new();
    let table_handle_state11 = TableHandleStateObject::new(
        handle11,
        Arc::new(storage11),
        *SPARSE_MERKLE_PLACEHOLDER_HASH,
    );
    let table_handle_state12 = TableHandleStateObject::new(
        handle12,
        Arc::new(storage12),
        *SPARSE_MERKLE_PLACEHOLDER_HASH,
    );
    let table_handle_state21 = TableHandleStateObject::new(
        handle11,
        Arc::new(storage21),
        *SPARSE_MERKLE_PLACEHOLDER_HASH,
    );
    let table_handle_state22 = TableHandleStateObject::new(
        handle12,
        Arc::new(storage22),
        *SPARSE_MERKLE_PLACEHOLDER_HASH,
    );
    let table_handle_state31 = TableHandleStateObject::new(
        handle11,
        Arc::new(storage31),
        *SPARSE_MERKLE_PLACEHOLDER_HASH,
    );
    let table_handle_state32 = TableHandleStateObject::new(
        handle12,
        Arc::new(storage32),
        *SPARSE_MERKLE_PLACEHOLDER_HASH,
    );
    let table_handle_state41 = TableHandleStateObject::new(
        handle11,
        Arc::new(storage41),
        *SPARSE_MERKLE_PLACEHOLDER_HASH,
    );
    let table_handle_state42 = TableHandleStateObject::new(
        handle12,
        Arc::new(storage42),
        *SPARSE_MERKLE_PLACEHOLDER_HASH,
    );

    table_handle_state11.set(key11, val11);
    table_handle_state11.set(key13, val13);
    table_handle_state12.set(key12, val12);
    table_handle_state12.set(key14, val14);
    table_handle_state21.set(key21, val21);
    table_handle_state21.set(key23, val23);
    table_handle_state22.set(key22, val22);
    table_handle_state22.set(key24, val24);
    table_handle_state31.set(key31, val31);
    table_handle_state31.set(key33, val33);
    table_handle_state32.set(key32, val32);
    table_handle_state32.set(key34, val34);
    table_handle_state41.set(key41, val41);
    table_handle_state41.set(key43, val43);
    table_handle_state42.set(key42, val42);
    table_handle_state42.set(key44, val44);
    table_handle_state11.commit()?;
    table_handle_state11.flush()?;
    table_handle_state12.commit()?;
    table_handle_state12.flush()?;
    table_handle_state21.commit()?;
    table_handle_state21.flush()?;
    table_handle_state22.commit()?;
    table_handle_state22.flush()?;
    table_handle_state31.commit()?;
    table_handle_state31.flush()?;
    table_handle_state32.commit()?;
    table_handle_state32.flush()?;
    table_handle_state41.commit()?;
    table_handle_state41.flush()?;
    table_handle_state42.commit()?;
    table_handle_state42.flush()?;

    let state_tree_table_handles1 = StateTree::new(Arc::new(storage1), None);
    state_tree_table_handles1.put(handle11, table_handle_state11.root_hash().to_vec());
    state_tree_table_handles1.put(handle12, table_handle_state12.root_hash().to_vec());
    state_tree_table_handles1.commit()?;
    state_tree_table_handles1.flush()?;

    let state_tree_table_handles2 = StateTree::new(Arc::new(storage2), None);
    state_tree_table_handles2.put(handle21, table_handle_state21.root_hash().to_vec());
    state_tree_table_handles2.put(handle22, table_handle_state22.root_hash().to_vec());
    state_tree_table_handles2.commit()?;
    state_tree_table_handles2.flush()?;

    let state_tree_table_handles3 = StateTree::new(Arc::new(storage3), None);
    state_tree_table_handles3.put(handle31, table_handle_state31.root_hash().to_vec());
    state_tree_table_handles3.put(handle32, table_handle_state32.root_hash().to_vec());
    state_tree_table_handles3.commit()?;
    state_tree_table_handles3.flush()?;

    let state_tree_table_handles4 = StateTree::new(Arc::new(storage4), None);
    state_tree_table_handles4.put(handle41, table_handle_state41.root_hash().to_vec());
    state_tree_table_handles4.put(handle42, table_handle_state42.root_hash().to_vec());
    state_tree_table_handles4.commit()?;
    state_tree_table_handles4.flush()?;

    assert_eq!(
        chain_state_db.table_handle_address_root_hash(0),
        state_tree_table_handles1.root_hash()
    );
    assert_eq!(
        chain_state_db.table_handle_address_root_hash(1),
        state_tree_table_handles2.root_hash()
    );
    assert_eq!(
        chain_state_db.table_handle_address_root_hash(2),
        state_tree_table_handles3.root_hash()
    );
    assert_eq!(
        chain_state_db.table_handle_address_root_hash(3),
        state_tree_table_handles4.root_hash()
    );

    assert_eq!(
        chain_state_db.table_handles_root_hash(0).unwrap(),
        state_tree_table_handles1.root_hash()
    );
    assert_eq!(
        chain_state_db.table_handles_root_hash(1).unwrap(),
        state_tree_table_handles2.root_hash()
    );
    assert_eq!(
        chain_state_db.table_handles_root_hash(2).unwrap(),
        state_tree_table_handles3.root_hash()
    );
    assert_eq!(
        chain_state_db.table_handles_root_hash(3).unwrap(),
        state_tree_table_handles4.root_hash()
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
    let account_address1 = AccountAddress::random();
    let module1 = random_identity();
    let state_key1 = StateKey::module(&account_address1, &module1);
    let account_address2 = AccountAddress::random();
    let struct_tag2 = StructTag {
        address: account_address2,
        module: random_identity(),
        name: random_identity(),
        type_args: vec![],
    };
    let state_key2 = StateKey::resource(&account_address2, &struct_tag2);
    let state_keys = vec![
        state_key1,
        state_key2,
        StateKey::table_item(&handle1, key1.as_slice()),
        StateKey::table_item(&handle1, key2.as_slice()),
        StateKey::table_item(&handle2, key3.as_slice()),
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
