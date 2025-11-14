use super::*;
use starcoin_crypto::HashValue;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::vm_error::KeptVMStatus;
use starcoin_vm2_vm_types::account_address::AccountAddress;
use starcoin_vm2_vm_types::identifier::{IdentStr, Identifier};
use starcoin_vm2_vm_types::language_storage::StructTag;
use starcoin_vm2_vm_types::state_store::table::TableHandle;
use starcoin_vm2_vm_types::state_store::TStateView;
use starcoin_vm2_vm_types::write_set::{WriteOp as WSWriteOp, WriteSetMut};

fn apply_op(db: &ChainStateDB, key: StateKey, op: WSWriteOp) {
    let ws = WriteSetMut::new(vec![(key, op)]);
    let frozen = ws.freeze().expect("freeze write set");
    db.apply_write_set(frozen).expect("apply write set");
    let _ = db.commit().expect("commit state");
}

#[test]
fn plan_merge_reuse_and_apply() {
    let state_db = ChainStateDB::mock();
    let handle = TableHandle(AccountAddress::new([1u8; 16]));
    let k1 = StateKey::table_item(&handle, b"k1");
    let v1 = b"v1".to_vec();
    apply_op(
        &state_db,
        k1.clone(),
        WSWriteOp::legacy_creation(v1.clone().into()),
    );
    let stored = TStateView::get_state_value(&state_db, &k1)
        .unwrap()
        .unwrap();
    let stored_hash = stored.hash();

    let tx_hash = HashValue::sha3_256_of(b"tx1");

    // ExecRecord: read K1==v1
    let rec = ExecRecord {
        tx_hash,
        epoch_id: 0,
        base_state_root: Some(HashValue::zero()),
        read_set: Some(vec![ReadEntry {
            key: k1.clone(),
            from_storage: true,
            existed: true,
            value_hash: stored_hash,
        }]),
        write_set: vec![],
        event_root: HashValue::zero(),
        gas: 1,
        status_ok: true,
        meta_fingerprint: None,
        status: Some(KeptVMStatus::Executed),
        events: Vec::new(),
        table_infos: Vec::new(),
    };

    let eng = MergeEngine::new();
    let mut prefix = PrefixWrites::default();
    let diff = eng.plan_merge(&state_db, &mut prefix, &[rec.clone()]);
    assert_eq!(diff.reexec.len(), 0);
    assert_eq!(diff.reused, vec![tx_hash]);
    assert_eq!(diff.writes.len(), 0);
}

#[test]
fn plan_merge_detect_prefix_conflict() {
    let handle = TableHandle(AccountAddress::new([3u8; 16]));
    let kx = StateKey::table_item(&handle, b"k");
    // prefix already wrote kx
    let mut prefix = PrefixWrites::default();
    prefix.0.insert(kx.clone());

    let tx_hash = HashValue::sha3_256_of(b"tx2");
    let rec = ExecRecord {
        tx_hash,
        epoch_id: 0,
        base_state_root: Some(HashValue::zero()),
        read_set: Some(vec![ReadEntry {
            key: kx.clone(),
            from_storage: true,
            existed: false,
            value_hash: HashValue::zero(),
        }]),
        write_set: vec![],
        event_root: HashValue::zero(),
        gas: 1,
        status_ok: true,
        meta_fingerprint: None,
        status: Some(KeptVMStatus::Executed),
        events: Vec::new(),
        table_infos: Vec::new(),
    };
    let eng = MergeEngine::new();
    let diff = eng.plan_merge(&ChainStateDB::mock(), &mut prefix, &[rec]);
    assert_eq!(diff.reexec, vec![tx_hash]);
    assert_eq!(diff.reused.len(), 0);
}

#[test]
fn plan_merge_detect_value_change() {
    let state_db = ChainStateDB::mock();
    let handle = TableHandle(AccountAddress::new([4u8; 16]));
    let k1 = StateKey::table_item(&handle, b"k1");
    let v1 = b"v1".to_vec();
    apply_op(
        &state_db,
        k1.clone(),
        WSWriteOp::legacy_creation(v1.clone().into()),
    );
    apply_op(
        &state_db,
        k1.clone(),
        WSWriteOp::legacy_modification(b"v1b".to_vec().into()),
    );

    let tx_hash = HashValue::sha3_256_of(b"tx3");
    let rec = ExecRecord {
        tx_hash,
        epoch_id: 0,
        base_state_root: Some(HashValue::zero()),
        read_set: Some(vec![ReadEntry {
            key: k1.clone(),
            from_storage: true,
            existed: true,
            value_hash: HashValue::sha3_256_of(&v1),
        }]),
        write_set: vec![],
        event_root: HashValue::zero(),
        gas: 1,
        status_ok: true,
        meta_fingerprint: None,
        status: Some(KeptVMStatus::Executed),
        events: Vec::new(),
        table_infos: Vec::new(),
    };
    let mut prefix = PrefixWrites::default();
    let eng = MergeEngine::new();
    let diff = eng.plan_merge(&state_db, &mut prefix, &[rec]);
    assert_eq!(diff.reexec, vec![tx_hash]);
}

// -------- apply_diff complex scenarios (with real statedb-v2) --------
#[test]
fn apply_diff_empty_is_noop() {
    let statedb = ChainStateDB::mock();
    let eng = MergeEngine::new();
    let diff = MergeDiff::default();
    let pre_root = statedb.state_root();
    let res = eng.apply_diff(&statedb, &diff).expect("apply succeeds");
    assert_eq!(res.applied, 0);
    assert_eq!(res.state_root2, pre_root);
}

#[test]
fn apply_diff_create_modify_delete_table_item() {
    let statedb = ChainStateDB::mock();
    let eng = MergeEngine::new();

    let handle = TableHandle(AccountAddress::new([7u8; 16]));
    let k = StateKey::table_item(&handle, b"k");

    // 1) create
    let mut diff = MergeDiff::default();
    diff.writes
        .push((k.clone(), WSWriteOp::legacy_creation(b"v1".to_vec().into())));
    let r1 = eng.apply_diff(&statedb, &diff).expect("apply create");
    assert!(r1.applied > 0);
    let v = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(v.bytes().as_ref(), b"v1");

    // 2) modify
    let mut diff2 = MergeDiff::default();
    diff2.writes.push((
        k.clone(),
        WSWriteOp::legacy_modification(b"v2".to_vec().into()),
    ));
    let r2 = eng.apply_diff(&statedb, &diff2).expect("apply modify");
    assert_ne!(r1.state_root2, r2.state_root2);
    let v2 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(v2.bytes().as_ref(), b"v2");

    // 3) delete
    let mut diff3 = MergeDiff::default();
    diff3.writes.push((k.clone(), WSWriteOp::legacy_deletion()));
    let r3 = eng.apply_diff(&statedb, &diff3).expect("apply delete");
    assert_ne!(r2.state_root2, r3.state_root2);
    let v3 = TStateView::get_state_value(&statedb, &k).unwrap();
    assert!(v3.is_none());
}

#[test]
fn apply_diff_no_commit_returns_none_for_empty_diff() {
    let eng = MergeEngine::new();
    let diff = MergeDiff::default();
    let res = eng
        .apply_diff_no_commit(&diff)
        .expect("no_commit should not fail");
    assert!(res.is_none());
}

#[test]
fn apply_diff_no_commit_materializes_writes() {
    let eng = MergeEngine::new();
    let handle = TableHandle(AccountAddress::new([0xAA; 16]));
    let k1 = StateKey::table_item(&handle, b"k1");
    let k2 = StateKey::table_item(&handle, b"k2");

    let mut diff = MergeDiff::default();
    diff.writes.push((
        k1.clone(),
        WSWriteOp::legacy_creation(b"v1".to_vec().into()),
    ));
    diff.writes.push((
        k2.clone(),
        WSWriteOp::legacy_modification(b"v2".to_vec().into()),
    ));

    let frozen = eng
        .apply_diff_no_commit(&diff)
        .expect("no_commit succeeds")
        .expect("writes should materialize");
    let expected = WriteSetMut::new(diff.writes.clone())
        .freeze()
        .expect("freeze succeeds");
    assert_eq!(frozen, expected);
}

#[test]
fn apply_diff_duplicate_key_last_wins() {
    let statedb = ChainStateDB::mock();
    let eng = MergeEngine::new();

    let handle = TableHandle(AccountAddress::new([9u8; 16]));
    let k = StateKey::table_item(&handle, b"dup");

    // Same key appears twice; last should win in WriteSetMut::new (BTreeMap collect).
    let mut diff = MergeDiff::default();
    diff.writes
        .push((k.clone(), WSWriteOp::legacy_creation(b"v1".to_vec().into())));
    diff.writes.push((
        k.clone(),
        WSWriteOp::legacy_modification(b"v2".to_vec().into()),
    ));
    let _ = eng.apply_diff(&statedb, &diff).expect("apply succeeds");
    let v = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(v.bytes().as_ref(), b"v2");
}

#[test]
fn apply_diff_multi_handles_independent_updates() {
    let statedb = ChainStateDB::mock();
    let eng = MergeEngine::new();

    let h1 = TableHandle(AccountAddress::new([0x11; 16]));
    let h2 = TableHandle(AccountAddress::new([0x22; 16]));
    let k1 = StateKey::table_item(&h1, b"a");
    let k2 = StateKey::table_item(&h2, b"b");

    let mut diff = MergeDiff::default();
    diff.writes
        .push((k1.clone(), WSWriteOp::legacy_creation(b"1".to_vec().into())));
    diff.writes
        .push((k2.clone(), WSWriteOp::legacy_creation(b"2".to_vec().into())));
    let _ = eng.apply_diff(&statedb, &diff).expect("apply multi");

    let v1 = TStateView::get_state_value(&statedb, &k1).unwrap().unwrap();
    let v2 = TStateView::get_state_value(&statedb, &k2).unwrap().unwrap();
    assert_eq!(v1.bytes().as_ref(), b"1");
    assert_eq!(v2.bytes().as_ref(), b"2");

    // Follow-up update just to k2 shouldn't affect k1 value
    let mut diff2 = MergeDiff::default();
    diff2.writes.push((
        k2.clone(),
        WSWriteOp::legacy_modification(b"22".to_vec().into()),
    ));
    let _ = eng.apply_diff(&statedb, &diff2).expect("apply modify h2");

    let v1b = TStateView::get_state_value(&statedb, &k1).unwrap().unwrap();
    let v2b = TStateView::get_state_value(&statedb, &k2).unwrap().unwrap();
    assert_eq!(v1b.bytes().as_ref(), b"1");
    assert_eq!(v2b.bytes().as_ref(), b"22");
}

#[test]
fn apply_diff_resource_crud() {
    let statedb = ChainStateDB::mock();
    let eng = MergeEngine::new();

    let addr = AccountAddress::new([0xAB; 16]);
    let tag = StructTag {
        address: addr,
        module: Identifier::new("M").unwrap(),
        name: Identifier::new("T").unwrap(),
        type_args: vec![],
    };
    let k = StateKey::resource(&addr, &tag).unwrap();

    // create
    let mut d1 = MergeDiff::default();
    d1.writes
        .push((k.clone(), WSWriteOp::legacy_creation(b"r1".to_vec().into())));
    let r1 = eng.apply_diff(&statedb, &d1).expect("create");
    let v1 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(v1.bytes().as_ref(), b"r1");

    // modify
    let mut d2 = MergeDiff::default();
    d2.writes.push((
        k.clone(),
        WSWriteOp::legacy_modification(b"r2".to_vec().into()),
    ));
    let r2 = eng.apply_diff(&statedb, &d2).expect("modify");
    assert_ne!(r1.state_root2, r2.state_root2);
    let v2 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(v2.bytes().as_ref(), b"r2");

    // delete
    let mut d3 = MergeDiff::default();
    d3.writes.push((k.clone(), WSWriteOp::legacy_deletion()));
    let r3 = eng.apply_diff(&statedb, &d3).expect("delete");
    assert_ne!(r2.state_root2, r3.state_root2);
    assert!(TStateView::get_state_value(&statedb, &k).unwrap().is_none());
}

#[test]
fn apply_diff_resource_group_crud() {
    let statedb = ChainStateDB::mock();
    let eng = MergeEngine::new();

    let addr = AccountAddress::new([0xCD; 16]);
    let tag = StructTag {
        address: addr,
        module: Identifier::new("G").unwrap(),
        name: Identifier::new("RG").unwrap(),
        type_args: vec![],
    };
    let k = StateKey::resource_group(&addr, &tag);

    // create
    let mut d1 = MergeDiff::default();
    d1.writes
        .push((k.clone(), WSWriteOp::legacy_creation(b"g1".to_vec().into())));
    let r1 = eng.apply_diff(&statedb, &d1).expect("create rg");
    let v1 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(v1.bytes().as_ref(), b"g1");

    // modify
    let mut d2 = MergeDiff::default();
    d2.writes.push((
        k.clone(),
        WSWriteOp::legacy_modification(b"g2".to_vec().into()),
    ));
    let r2 = eng.apply_diff(&statedb, &d2).expect("modify rg");
    assert_ne!(r1.state_root2, r2.state_root2);
    let v2 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(v2.bytes().as_ref(), b"g2");

    // delete
    let mut d3 = MergeDiff::default();
    d3.writes.push((k.clone(), WSWriteOp::legacy_deletion()));
    let r3 = eng.apply_diff(&statedb, &d3).expect("delete rg");
    assert_ne!(r2.state_root2, r3.state_root2);
    assert!(TStateView::get_state_value(&statedb, &k).unwrap().is_none());
}

#[test]
fn apply_diff_module_create_modify_delete_fails() {
    let statedb = ChainStateDB::mock();
    let eng = MergeEngine::new();

    let addr = AccountAddress::new([0xEF; 16]);
    let name: &IdentStr = IdentStr::new("Mod").unwrap();
    let k = StateKey::module(&addr, name);

    // create module
    let mut d1 = MergeDiff::default();
    d1.writes
        .push((k.clone(), WSWriteOp::legacy_creation(b"m1".to_vec().into())));
    let _ = eng.apply_diff(&statedb, &d1).expect("module create");
    let m1 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(m1.bytes().as_ref(), b"m1");

    // modify module
    let mut d2 = MergeDiff::default();
    d2.writes.push((
        k.clone(),
        WSWriteOp::legacy_modification(b"m2".to_vec().into()),
    ));
    let _ = eng.apply_diff(&statedb, &d2).expect("module modify");
    let m2 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(m2.bytes().as_ref(), b"m2");

    // delete module should fail (not supported)
    let mut d3 = MergeDiff::default();
    d3.writes.push((k.clone(), WSWriteOp::legacy_deletion()));
    assert!(eng.apply_diff(&statedb, &d3).is_err());

    // still present after failed deletion
    let m2b = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(m2b.bytes().as_ref(), b"m2");
}

#[test]
fn apply_diff_idempotent_same_value_no_root_change() {
    let statedb = ChainStateDB::mock();
    let eng = MergeEngine::new();

    let handle = TableHandle(AccountAddress::new([0x44; 16]));
    let k = StateKey::table_item(&handle, b"idem");

    // initial create
    let mut d1 = MergeDiff::default();
    d1.writes.push((
        k.clone(),
        WSWriteOp::legacy_creation(b"same".to_vec().into()),
    ));
    let r1 = eng.apply_diff(&statedb, &d1).expect("create");

    // apply same value again
    let mut d2 = MergeDiff::default();
    d2.writes.push((
        k.clone(),
        WSWriteOp::legacy_modification(b"same".to_vec().into()),
    ));
    let r2 = eng.apply_diff(&statedb, &d2).expect("modify same");

    // root may remain unchanged if underlying tree detects no diff
    assert_eq!(r1.state_root2, r2.state_root2);
    let v = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
    assert_eq!(v.bytes().as_ref(), b"same");
}

#[test]
fn apply_diff_delete_nonexistent_resource_no_root_change() {
    let statedb = ChainStateDB::mock();
    let eng = MergeEngine::new();

    let addr = AccountAddress::new([0x33; 16]);
    let tag = StructTag {
        address: addr,
        module: Identifier::new("M").unwrap(),
        name: Identifier::new("T").unwrap(),
        type_args: vec![],
    };
    let k = StateKey::resource(&addr, &tag).unwrap();

    // baseline root
    let mut d = MergeDiff::default();
    d.writes.push((k.clone(), WSWriteOp::legacy_deletion()));
    let res = eng.apply_diff(&statedb, &d);
    assert!(res.is_err(), "deleting non-existent resource should err");
    // still nonexistent
    assert!(TStateView::get_state_value(&statedb, &k).unwrap().is_none());
}
