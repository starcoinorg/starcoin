use anyhow::Result;
use once_cell::sync::Lazy;
use quick_cache::sync::Cache;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::CryptoHash;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_vm2_statedb::{ChainStateDB, ChainStateReader, ChainStateWriter};
use starcoin_vm2_types::contract_event::ContractEvent;
use starcoin_vm2_types::vm_error::KeptVMStatus;
use starcoin_vm2_vm_types::state_store::errors::StateviewError;
use starcoin_vm2_vm_types::state_store::state_key::StateKey;
use starcoin_vm2_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm2_vm_types::state_store::TStateView;
use starcoin_vm2_vm_types::write_set::WriteOp;
use starcoin_vm2_vm_types::write_set::{WriteSet, WriteSetMut};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

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
    pub epoch_id: u64,
    #[serde(default)]
    pub base_state_root: Option<HashValue>,
    pub read_set: Option<Vec<ReadEntry>>, // None means unknown -> force reexec
    pub write_set: Vec<(StateKey, WriteOp)>,
    pub event_root: HashValue,
    pub gas: u64,
    pub status_ok: bool,
    pub meta_fingerprint: Option<HashValue>,
    #[serde(default)]
    pub status: Option<KeptVMStatus>,
    #[serde(default)]
    pub events: Vec<ContractEvent>,
    #[serde(default)]
    pub table_infos: Vec<(TableHandle, TableInfo)>,
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

pub fn hydrate_read_set_for_writes<S>(
    state: &S,
    read_set: &mut Option<Vec<ReadEntry>>,
    writes: &[(StateKey, WriteOp)],
) -> std::result::Result<(), StateviewError>
where
    S: TStateView<Key = StateKey>,
{
    if writes.is_empty() {
        return Ok(());
    }

    let entries = read_set.get_or_insert_with(Vec::new);
    let mut seen: HashSet<StateKey> = entries.iter().map(|entry| entry.key.clone()).collect();

    for (key, op) in writes.iter() {
        if !seen.insert(key.clone()) {
            continue;
        }

        match op {
            WriteOp::Creation { .. } => {
                entries.push(ReadEntry {
                    key: key.clone(),
                    from_storage: true,
                    existed: false,
                    value_hash: HashValue::zero(),
                });
            }
            WriteOp::Modification { .. } | WriteOp::Deletion { .. } => {
                let maybe_value = state.get_state_value(key)?;
                let (existed, value_hash) = match maybe_value {
                    Some(state_value) => (true, state_value.hash()),
                    None => (false, HashValue::zero()),
                };
                entries.push(ReadEntry {
                    key: key.clone(),
                    from_storage: true,
                    existed,
                    value_hash,
                });
            }
        }
    }

    Ok(())
}

// ------------------------------
// State view for value-hash reads
// ------------------------------

pub trait StateViewExt {
    fn get_value_hash(&self, key: &StateKey) -> Option<HashValue>;
}

impl StateViewExt for ChainStateDB {
    fn get_value_hash(&self, key: &StateKey) -> Option<HashValue> {
        self.get_state_value(key)
            .ok()
            .and_then(|maybe_value| maybe_value.map(|state_value| state_value.hash()))
    }
}

// ------------------------------
// Witness Store (LRU)
// ------------------------------

#[derive(Clone, Debug, Eq)]
pub struct ExecKey {
    pub tx_hash: HashValue,
    pub epoch_id: u64,
}

impl PartialEq for ExecKey {
    fn eq(&self, other: &Self) -> bool {
        self.tx_hash == other.tx_hash && self.epoch_id == other.epoch_id
    }
}
impl Hash for ExecKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tx_hash.hash(state);
        self.epoch_id.hash(state);
    }
}

pub trait WitnessStore: Send + Sync {
    fn get(&self, key: &ExecKey) -> Option<ExecRecord>;
    fn put(&self, rec: ExecRecord);
    fn remove(&self, key: &ExecKey);
}

pub struct LruWitnessStore {
    cache: Cache<ExecKey, ExecRecord>,
}

impl LruWitnessStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Cache::new(capacity),
        }
    }

    pub fn clear(&self) {
        self.cache.clear();
    }
}

impl WitnessStore for LruWitnessStore {
    fn get(&self, key: &ExecKey) -> Option<ExecRecord> {
        self.cache.get(key)
    }
    fn put(&self, rec: ExecRecord) {
        let key = ExecKey {
            tx_hash: rec.tx_hash,
            epoch_id: rec.epoch_id,
        };
        self.cache.insert(key, rec);
    }
    fn remove(&self, key: &ExecKey) {
        self.cache.remove(key);
    }
}

#[derive(Clone, Default)]
pub struct MergeEngine {}

impl MergeEngine {
    pub fn new() -> Self {
        Self::default()
    }

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
                        info!(
                            "plan_merge prefix_conflict tx={} key={:?}",
                            rec.tx_hash, r.key
                        );
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
                        info!(
                            "plan_merge hash_mismatch tx={} key={:?} expected_exists={} expected_hash={} actual={:?}",
                            rec.tx_hash,
                            r.key,
                            r.existed,
                            r.value_hash,
                            cur
                        );
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

    fn materialize_write_set(&self, diff: &MergeDiff) -> Result<Option<WriteSet>> {
        if diff.writes.is_empty() {
            return Ok(None);
        }
        let ws = WriteSetMut::new(diff.writes.clone()).freeze()?;
        Ok(Some(ws))
    }

    /// Returns a frozen WriteSet ready for deferred application without mutating storage.
    pub fn apply_diff_no_commit(&self, diff: &MergeDiff) -> Result<Option<WriteSet>> {
        self.materialize_write_set(diff)
    }

    pub fn apply_diff(&self, state_db: &ChainStateDB, diff: &MergeDiff) -> Result<ApplyResult> {
        if let Some(ws) = self.materialize_write_set(diff)? {
            state_db.apply_write_set(ws)?;
            let root = state_db.commit()?;
            return Ok(ApplyResult {
                state_root2: root,
                applied: diff.writes.len(),
            });
        }
        Ok(ApplyResult {
            state_root2: state_db.state_root(),
            applied: 0,
        })
    }
}

// ------------------------------
// Reuse options & helpers
// ------------------------------

#[derive(Clone)]
pub struct ReuseOpts {
    pub enabled: bool,
    pub epoch_id: u64,
    pub base_state_root: Option<HashValue>,
    pub witness_store: Arc<dyn WitnessStore>,
    pub merge_engine: Arc<MergeEngine>,
}

pub fn create_default_reuse(enabled: bool, epoch_id: u64, capacity: usize) -> ReuseOpts {
    ReuseOpts {
        enabled,
        epoch_id,
        base_state_root: None,
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

/// Clears the global witness cache. Intended for test environments.
pub fn reset_global_witness_store_for_tests() {
    GLOBAL_WITNESS_STORE.clear();
}

pub fn build_prefix_from_writes(writes: &[(StateKey, WriteOp)]) -> PrefixWrites {
    let mut set = HashSet::with_capacity(writes.len());
    for (k, _) in writes.iter() {
        set.insert(k.clone());
    }
    PrefixWrites(set)
}

mod reuse;
pub use reuse::{
    execute_transactions_with_reuse, reset_reuse_counters_for_test, reuse_counters_for_test,
    PlanDecision, PlanEntry, PlanStats, ReusePlan,
};

#[cfg(test)]
mod tests;
