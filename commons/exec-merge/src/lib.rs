use anyhow::Result;
use quick_cache::sync::Cache;
use serde::{Deserialize, Serialize};
use starcoin_vm2_crypto::HashValue;
#[cfg(any(feature = "statedb", test))]
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_vm_types::state_store::state_key::StateKey;
use starcoin_vm2_vm_types::write_set::{WriteOp, WriteSetMut};
#[cfg(any(feature = "statedb", test))]
use starcoin_vm2_statedb::{ChainStateReader, ChainStateWriter};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use once_cell::sync::Lazy;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadEntry {
    pub key: StateKey,
    pub from_storage: bool,
    pub existed: bool,
    pub value_hash: HashValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecRecord {
    pub tx_hash: HashValue,
    pub pre_state_fingerprint: HashValue,
    pub read_set: Option<Vec<ReadEntry>>, // None means unknown -> force reexec
    pub write_set: Vec<(StateKey, WriteOp)>,
    pub event_root: HashValue,
    pub gas: u64,
    pub status_ok: bool,
    pub meta_fingerprint: Option<HashValue>,
}

#[derive(Clone, Debug, Default)]
pub struct PrefixWrites(pub HashSet<StateKey>);

impl PrefixWrites {
    pub fn insert_all<I: IntoIterator<Item = StateKey>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
    pub fn contains(&self, k: &StateKey) -> bool {
        self.0.contains(k)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MergeDiff {
    pub writes: Vec<(StateKey, WriteOp)>,
    pub reused: Vec<HashValue>,
    pub reexec: Vec<HashValue>,
    pub stats: HashMap<&'static str, u64>,
}

#[derive(Clone, Debug, Default)]
pub struct ApplyResult {
    pub state_root2: HashValue,
    pub applied: usize,
}

// ------------------------------
// State view for value-hash reads
// ------------------------------

pub trait StateViewExt {
    fn get_value_hash(&self, key: &StateKey) -> Option<HashValue>;
}

// Default impls for production are provided elsewhere when integrating.

// ------------------------------
// Witness Store (LRU)
// ------------------------------

#[derive(Clone, Debug, Eq)]
pub struct ExecKey {
    pub tx_hash: HashValue,
    pub pre_state_fingerprint: HashValue,
}

impl PartialEq for ExecKey {
    fn eq(&self, other: &Self) -> bool {
        self.tx_hash == other.tx_hash && self.pre_state_fingerprint == other.pre_state_fingerprint
    }
}
impl Hash for ExecKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tx_hash.hash(state);
        self.pre_state_fingerprint.hash(state);
    }
}

pub trait WitnessStore: Send + Sync {
    fn get(&self, key: &ExecKey) -> Option<ExecRecord>;
    fn put(&self, rec: ExecRecord);
}

pub struct LruWitnessStore {
    cap: usize,
    cache: Cache<ExecKey, ExecRecord>,
}

impl LruWitnessStore {
    pub fn new(capacity: usize) -> Self {
        Self { cap: capacity, cache: Cache::new(capacity) }
    }
}

impl WitnessStore for LruWitnessStore {
    fn get(&self, key: &ExecKey) -> Option<ExecRecord> {
        self.cache.get(key)
    }
    fn put(&self, rec: ExecRecord) {
        let key = ExecKey { tx_hash: rec.tx_hash, pre_state_fingerprint: rec.pre_state_fingerprint };
        self.cache.insert(key, rec);
    }
}

#[derive(Clone, Default)]
pub struct MergeEngine {}

impl MergeEngine {
    pub fn new() -> Self { Self::default() }

    pub fn plan_merge(
        &self,
        state: &dyn StateViewExt,
        prefix: &mut PrefixWrites,
        execs: &[ExecRecord],
    ) -> MergeDiff {
        let mut diff = MergeDiff::default();
        let mut read_checked = 0u64;
        for rec in execs {
            let mut need_reexec = rec.read_set.is_none();
            if let Some(rs) = &rec.read_set {
                for r in rs.iter() {
                    read_checked += 1;
                    if prefix.contains(&r.key) {
                        need_reexec = true;
                        break;
                    }
                    // storage value/hash existence check
                    let cur = state.get_value_hash(&r.key);
                    let ok = if r.existed {
                        cur == Some(r.value_hash)
                    } else {
                        cur.is_none()
                    };
                    if !ok {
                        need_reexec = true;
                        break;
                    }
                }
            }
            if need_reexec {
                diff.reexec.push(rec.tx_hash);
            } else {
                diff.reused.push(rec.tx_hash);
                // append writes & update prefix
                for (k, v) in rec.write_set.iter() {
                    diff.writes.push((k.clone(), v.clone()));
                    prefix.0.insert(k.clone());
                }
            }
        }
        diff.stats.insert("read_checked", read_checked);
        diff.stats.insert("reused_count", diff.reused.len() as u64);
        diff.stats.insert("reexec_count", diff.reexec.len() as u64);
        diff
    }

    #[cfg(any(feature = "statedb", test))]
    pub fn apply_diff(&self, state_db: &ChainStateDB2, diff: &MergeDiff) -> Result<ApplyResult> {
        if diff.writes.is_empty() {
            return Ok(ApplyResult { state_root2: state_db.state_root(), applied: 0 });
        }
        // Build a WriteSet and apply.
        let ws = WriteSetMut::new(diff.writes.clone()).freeze()?;
        state_db.apply_write_set(ws)?;
        let root = state_db.commit()?;
        Ok(ApplyResult { state_root2: root, applied: diff.writes.len() })
    }
}

// ------------------------------
// Reuse options & helpers
// ------------------------------

#[derive(Clone)]
pub struct ReuseOpts {
    pub enabled: bool,
    pub pre_state_fingerprint: HashValue,
    pub witness_store: Arc<dyn WitnessStore>,
    pub merge_engine: Arc<MergeEngine>,
}

pub fn create_pre_state_fingerprint(
    parent_state_root2: HashValue,
    metadata_hash: HashValue,
    epoch_version: u64,
) -> HashValue {
    // simple mixing: sha3(parent || meta || epoch_u64_le)
    let mut buf = Vec::with_capacity(32 + 32 + 8);
    buf.extend_from_slice(parent_state_root2.as_ref());
    buf.extend_from_slice(metadata_hash.as_ref());
    buf.extend_from_slice(&epoch_version.to_le_bytes());
    HashValue::sha3_256_of(&buf)
}

pub fn create_default_reuse(
    enabled: bool,
    pre_state_fingerprint: HashValue,
    capacity: usize,
) -> ReuseOpts {
    ReuseOpts {
        enabled,
        pre_state_fingerprint,
        witness_store: Arc::new(LruWitnessStore::new(capacity)),
        merge_engine: Arc::new(MergeEngine::new()),
    }
}

// ------------------------------
// Global witness store (singleton)
// ------------------------------

static GLOBAL_WITNESS_STORE: Lazy<Arc<LruWitnessStore>> = Lazy::new(|| {
    // Fixed capacity to avoid env-based variability in tests
    const CAPACITY: usize = 100_000;
    Arc::new(LruWitnessStore::new(CAPACITY))
});

pub fn global_witness_store() -> Arc<dyn WitnessStore> {
    GLOBAL_WITNESS_STORE.clone() as Arc<dyn WitnessStore>
}

pub fn build_prefix_from_writes(writes: &[(StateKey, WriteOp)]) -> PrefixWrites {
    let mut set = HashSet::with_capacity(writes.len());
    for (k, _) in writes.iter() {
        set.insert(k.clone());
    }
    PrefixWrites(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_vm2_vm_types::state_store::table::TableHandle;
    use starcoin_vm2_vm_types::account_address::AccountAddress;
    use starcoin_vm2_vm_types::state_store::TStateView;
    use starcoin_vm2_vm_types::write_set::WriteOp as WSWriteOp;
    use starcoin_vm2_vm_types::language_storage::StructTag;
    use starcoin_vm2_vm_types::identifier::{IdentStr, Identifier};

    #[derive(Default)]
    struct MemState {
        // key -> Some(value_bytes) or None (absent)
        m: std::collections::HashMap<StateKey, Option<Vec<u8>>>,
    }

    impl StateViewExt for MemState {
        fn get_value_hash(&self, key: &StateKey) -> Option<HashValue> {
            match self.m.get(key) {
                Some(Some(v)) => Some(HashValue::sha3_256_of(v.as_slice())),
                Some(None) => None,
                None => None,
            }
        }
    }

    fn table_key(handle: TableHandle, raw: &[u8]) -> StateKey {
        StateKey::table_item(&handle, raw)
    }

    #[test]
    fn plan_merge_reuse_and_apply() {
        let mut state = MemState::default();
        // Prepare K1 with V1
        let handle = TableHandle(AccountAddress::new([1u8; 16]));
        let k1 = table_key(handle, b"k1");
        let v1 = b"v1".to_vec();
        state.m.insert(k1.clone(), Some(v1.clone()));

        let pre_fp = HashValue::sha3_256_of(b"pre");
        let tx_hash = HashValue::sha3_256_of(b"tx1");

        // ExecRecord: read K1==v1, write K2=v2
        let handle2 = TableHandle(AccountAddress::new([2u8; 16]));
        let k2 = table_key(handle2, b"k2");
        let v2 = b"v2".to_vec();
        let rec = ExecRecord {
            tx_hash,
            pre_state_fingerprint: pre_fp,
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
        };

        let eng = MergeEngine::new();
        let mut prefix = PrefixWrites::default();
        let diff = eng.plan_merge(&state, &mut prefix, &[rec.clone()]);
        assert_eq!(diff.reexec.len(), 0);
        assert_eq!(diff.reused, vec![tx_hash]);
        assert_eq!(diff.writes.len(), 0);
    }

    #[test]
    fn plan_merge_detect_prefix_conflict() {
        let state = MemState::default();
        let handle = TableHandle(AccountAddress::new([3u8; 16]));
        let kx = table_key(handle, b"k");
        // prefix already wrote kx
        let mut prefix = PrefixWrites::default();
        prefix.0.insert(kx.clone());

        let pre_fp = HashValue::sha3_256_of(b"pre2");
        let tx_hash = HashValue::sha3_256_of(b"tx2");
        let rec = ExecRecord {
            tx_hash,
            pre_state_fingerprint: pre_fp,
            read_set: Some(vec![ReadEntry { key: kx.clone(), from_storage: true, existed: false, value_hash: HashValue::zero() }]),
            write_set: vec![],
            event_root: HashValue::zero(),
            gas: 1,
            status_ok: true,
            meta_fingerprint: None,
        };
        let eng = MergeEngine::new();
        let diff = eng.plan_merge(&state, &mut prefix, &[rec]);
        assert_eq!(diff.reexec, vec![tx_hash]);
        assert_eq!(diff.reused.len(), 0);
    }

    #[test]
    fn plan_merge_detect_value_change() {
        let mut state = MemState::default();
        let handle = TableHandle(AccountAddress::new([4u8; 16]));
        let k1 = table_key(handle, b"k1");
        let v1 = b"v1".to_vec();
        // initial write
        state.m.insert(k1.clone(), Some(v1.clone()));
        // mutate before merge: write different value
        let v1b = b"v1b".to_vec();
        state.m.insert(k1.clone(), Some(v1b.clone()));

        let pre_fp = HashValue::sha3_256_of(b"pre3");
        let tx_hash = HashValue::sha3_256_of(b"tx3");
        let rec = ExecRecord {
            tx_hash,
            pre_state_fingerprint: pre_fp,
            read_set: Some(vec![ReadEntry { key: k1.clone(), from_storage: true, existed: true, value_hash: HashValue::sha3_256_of(&v1) }]),
            write_set: vec![],
            event_root: HashValue::zero(),
            gas: 1,
            status_ok: true,
            meta_fingerprint: None,
        };
        let mut prefix = PrefixWrites::default();
        let eng = MergeEngine::new();
        let diff = eng.plan_merge(&state, &mut prefix, &[rec]);
        assert_eq!(diff.reexec, vec![tx_hash]);
    }

    // -------- apply_diff complex scenarios (with real statedb-v2) --------
    #[test]
    fn apply_diff_empty_is_noop() {
        let statedb = ChainStateDB2::mock();
        let eng = MergeEngine::new();
        let diff = MergeDiff::default();
        let pre_root = statedb.state_root();
        let res = eng.apply_diff(&statedb, &diff).expect("apply succeeds");
        assert_eq!(res.applied, 0);
        assert_eq!(res.state_root2, pre_root);
    }

    #[test]
    fn apply_diff_create_modify_delete_table_item() {
        let statedb = ChainStateDB2::mock();
        let eng = MergeEngine::new();

        let handle = TableHandle(AccountAddress::new([7u8; 16]));
        let k = StateKey::table_item(&handle, b"k");

        // 1) create
        let mut diff = MergeDiff::default();
        diff.writes.push((k.clone(), WSWriteOp::legacy_creation(b"v1".to_vec().into())));
        let r1 = eng.apply_diff(&statedb, &diff).expect("apply create");
        assert!(r1.applied > 0);
        let v = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
        assert_eq!(v.bytes().as_ref(), b"v1");

        // 2) modify
        let mut diff2 = MergeDiff::default();
        diff2.writes.push((k.clone(), WSWriteOp::legacy_modification(b"v2".to_vec().into())));
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
    fn apply_diff_duplicate_key_last_wins() {
        let statedb = ChainStateDB2::mock();
        let eng = MergeEngine::new();

        let handle = TableHandle(AccountAddress::new([9u8; 16]));
        let k = StateKey::table_item(&handle, b"dup");

        // Same key appears twice; last should win in WriteSetMut::new (BTreeMap collect).
        let mut diff = MergeDiff::default();
        diff.writes.push((k.clone(), WSWriteOp::legacy_creation(b"v1".to_vec().into())));
        diff.writes.push((k.clone(), WSWriteOp::legacy_modification(b"v2".to_vec().into())));
        let _ = eng.apply_diff(&statedb, &diff).expect("apply succeeds");
        let v = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
        assert_eq!(v.bytes().as_ref(), b"v2");
    }

    #[test]
    fn apply_diff_multi_handles_independent_updates() {
        let statedb = ChainStateDB2::mock();
        let eng = MergeEngine::new();

        let h1 = TableHandle(AccountAddress::new([0x11; 16]));
        let h2 = TableHandle(AccountAddress::new([0x22; 16]));
        let k1 = StateKey::table_item(&h1, b"a");
        let k2 = StateKey::table_item(&h2, b"b");

        let mut diff = MergeDiff::default();
        diff.writes.push((k1.clone(), WSWriteOp::legacy_creation(b"1".to_vec().into())));
        diff.writes.push((k2.clone(), WSWriteOp::legacy_creation(b"2".to_vec().into())));
        let _ = eng.apply_diff(&statedb, &diff).expect("apply multi");

        let v1 = TStateView::get_state_value(&statedb, &k1).unwrap().unwrap();
        let v2 = TStateView::get_state_value(&statedb, &k2).unwrap().unwrap();
        assert_eq!(v1.bytes().as_ref(), b"1");
        assert_eq!(v2.bytes().as_ref(), b"2");

        // Follow-up update just to k2 shouldn't affect k1 value
        let mut diff2 = MergeDiff::default();
        diff2.writes.push((k2.clone(), WSWriteOp::legacy_modification(b"22".to_vec().into())));
        let _ = eng.apply_diff(&statedb, &diff2).expect("apply modify h2");

        let v1b = TStateView::get_state_value(&statedb, &k1).unwrap().unwrap();
        let v2b = TStateView::get_state_value(&statedb, &k2).unwrap().unwrap();
        assert_eq!(v1b.bytes().as_ref(), b"1");
        assert_eq!(v2b.bytes().as_ref(), b"22");
    }

    #[test]
    fn apply_diff_resource_crud() {
        let statedb = ChainStateDB2::mock();
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
        d1.writes.push((k.clone(), WSWriteOp::legacy_creation(b"r1".to_vec().into())));
        let r1 = eng.apply_diff(&statedb, &d1).expect("create");
        let v1 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
        assert_eq!(v1.bytes().as_ref(), b"r1");

        // modify
        let mut d2 = MergeDiff::default();
        d2.writes.push((k.clone(), WSWriteOp::legacy_modification(b"r2".to_vec().into())));
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
        let statedb = ChainStateDB2::mock();
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
        d1.writes.push((k.clone(), WSWriteOp::legacy_creation(b"g1".to_vec().into())));
        let r1 = eng.apply_diff(&statedb, &d1).expect("create rg");
        let v1 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
        assert_eq!(v1.bytes().as_ref(), b"g1");

        // modify
        let mut d2 = MergeDiff::default();
        d2.writes.push((k.clone(), WSWriteOp::legacy_modification(b"g2".to_vec().into())));
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
        let statedb = ChainStateDB2::mock();
        let eng = MergeEngine::new();

        let addr = AccountAddress::new([0xEF; 16]);
        let name: &IdentStr = IdentStr::new("Mod").unwrap();
        let k = StateKey::module(&addr, name);

        // create module
        let mut d1 = MergeDiff::default();
        d1.writes.push((k.clone(), WSWriteOp::legacy_creation(b"m1".to_vec().into())));
        let _ = eng.apply_diff(&statedb, &d1).expect("module create");
        let m1 = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
        assert_eq!(m1.bytes().as_ref(), b"m1");

        // modify module
        let mut d2 = MergeDiff::default();
        d2.writes.push((k.clone(), WSWriteOp::legacy_modification(b"m2".to_vec().into())));
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
        let statedb = ChainStateDB2::mock();
        let eng = MergeEngine::new();

        let handle = TableHandle(AccountAddress::new([0x44; 16]));
        let k = StateKey::table_item(&handle, b"idem");

        // initial create
        let mut d1 = MergeDiff::default();
        d1.writes.push((k.clone(), WSWriteOp::legacy_creation(b"same".to_vec().into())));
        let r1 = eng.apply_diff(&statedb, &d1).expect("create");

        // apply same value again
        let mut d2 = MergeDiff::default();
        d2.writes.push((k.clone(), WSWriteOp::legacy_modification(b"same".to_vec().into())));
        let r2 = eng.apply_diff(&statedb, &d2).expect("modify same");

        // root may remain unchanged if underlying tree detects no diff
        assert_eq!(r1.state_root2, r2.state_root2);
        let v = TStateView::get_state_value(&statedb, &k).unwrap().unwrap();
        assert_eq!(v.bytes().as_ref(), b"same");
    }

    #[test]
    fn apply_diff_delete_nonexistent_resource_no_root_change() {
        let statedb = ChainStateDB2::mock();
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
}
