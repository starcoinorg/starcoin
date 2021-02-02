use super::*;
use starcoin_state_tree::mock::MockStateNodeStore;
use starcoin_types::write_set::{WriteOp, WriteSet, WriteSetMut};
use starcoin_vm_types::account_config::AccountResource;
use starcoin_vm_types::move_resource::MoveResource;

fn random_bytes() -> Vec<u8> {
    HashValue::random().to_vec()
}

fn to_write_set(access_path: AccessPath, value: Vec<u8>) -> WriteSet {
    WriteSetMut::new(vec![(access_path, WriteOp::Value(value))])
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
    let state1 = chain_state_db.get(&access_path)?;
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
    let old_state2 = chain_state_db_ori.get(&access_path)?.unwrap();
    assert_eq!(old_state, old_state2);

    Ok(())
}
