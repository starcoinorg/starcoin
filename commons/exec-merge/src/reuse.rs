use crate::{ExecKey, ExecRecord, MergeDiff, MergeEngine, PrefixWrites, ReuseOpts, WitnessStore};
use starcoin_crypto::hash::CryptoHash;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_vm2_executor::{
    block_executor,
    block_executor::{BlockExecutedData, VMMetrics},
    do_execute_block_transactions,
};
use starcoin_vm2_statedb::{ChainStateDB, ChainStateReader, ChainStateWriter};
use starcoin_vm2_types::contract_event::ContractEvent;
use starcoin_vm2_types::error::{BlockExecutorError, ExecutorResult};
use starcoin_vm2_types::transaction::{Transaction, TransactionInfo, TransactionStatus};
use starcoin_vm2_types::vm_error::KeptVMStatus;
use starcoin_vm2_vm_types::state_store::state_key::{inner::StateKeyInner, StateKey};
use starcoin_vm2_vm_types::state_store::state_value::StateValue;
use starcoin_vm2_vm_types::state_store::TStateView;
use starcoin_vm2_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};
use starcoin_vm_runtime::{record_fee_payer_for_reuse, reuse_recorder};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug)]
pub struct PlanStats {
    pub total: usize,
    pub witness_hits: usize,
    pub witness_hits_with_reads: usize,
    pub read_checked: u64,
    pub plan_time_ms: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanDecision {
    Reuse,
    Reexec,
}

#[derive(Debug)]
pub struct PlanEntry {
    pub sanitized: ExecRecord,
    pub witness: Option<ExecRecord>,
    pub decision: PlanDecision,
    pub forced_reexec: bool,
    pub reusable: bool,
}

#[derive(Debug)]
pub struct ReusePlan {
    pub entries: Vec<PlanEntry>,
    pub stats: PlanStats,
}

impl ReusePlan {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn reused_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.decision == PlanDecision::Reuse)
            .count()
    }

    pub fn reexec_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.decision == PlanDecision::Reexec)
            .count()
    }
}

static TEST_REUSE_HITS: AtomicUsize = AtomicUsize::new(0);
static TEST_REUSE_REEXEC: AtomicUsize = AtomicUsize::new(0);

pub fn reset_reuse_counters_for_test() {
    TEST_REUSE_HITS.store(0, Ordering::Relaxed);
    TEST_REUSE_REEXEC.store(0, Ordering::Relaxed);
}

pub fn reuse_counters_for_test() -> (usize, usize) {
    (
        TEST_REUSE_HITS.load(Ordering::Relaxed),
        TEST_REUSE_REEXEC.load(Ordering::Relaxed),
    )
}

pub fn execute_transactions_with_reuse(
    statedb: &ChainStateDB,
    transactions: Vec<Transaction>,
    gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
    opts: ReuseOpts,
) -> ExecutorResult<BlockExecutedData> {
    let meta_fingerprint = extract_block_metadata_fingerprint(&transactions);
    let base_state_root = opts
        .base_state_root
        .unwrap_or_else(|| statedb.state_root());
    let executor = ReuseExecutor::new(
        statedb,
        opts.epoch_id,
        opts.witness_store.clone(),
        meta_fingerprint,
        base_state_root,
    );

    if opts.enabled {
        let planner = ReusePlanner::new(
            statedb,
            opts.witness_store.clone(),
            opts.merge_engine.clone(),
            opts.epoch_id,
            meta_fingerprint,
            base_state_root,
        );
        if let Some(plan) = planner.plan(&transactions) {
            let reused = plan.reused_count();
            let reexec = plan.reexec_count();
            info!(
                "execute_transactions_with_reuse reused={} reexec={}",
                reused, reexec
            );
            let stats = &plan.stats;

            TEST_REUSE_HITS.fetch_add(reused, Ordering::Relaxed);
            TEST_REUSE_REEXEC.fetch_add(reexec, Ordering::Relaxed);

            let witness_missing = stats.total.saturating_sub(stats.witness_hits);
            debug!(
                "vm2 reuse plan summary: total={}, witness_hits={}, witness_hits_with_reads={}, reused={}, reexec={}, missing={}, read_checked={}, plan_time_ms={}",
                stats.total,
                stats.witness_hits,
                stats.witness_hits_with_reads,
                reused,
                reexec,
                witness_missing,
                stats.read_checked,
                stats.plan_time_ms,
            );

            return executor.execute_plan(&transactions, gas_limit, vm_metrics, plan);
        }
    }

    executor.execute_full(transactions, gas_limit, vm_metrics, opts.enabled)
}

fn extract_block_metadata_fingerprint(transactions: &[Transaction]) -> Option<HashValue> {
    transactions.iter().find_map(|txn| match txn {
        Transaction::BlockMetadata(_) => Some(txn.id()),
        _ => None,
    })
}

struct ReusePlanner<'a> {
    statedb: &'a ChainStateDB,
    store: Arc<dyn WitnessStore>,
    merge_engine: Arc<MergeEngine>,
    epoch_id: u64,
    meta_fingerprint: Option<HashValue>,
    base_state_root: HashValue,
}

#[derive(Clone, Debug)]
struct CachedState {
    exists: bool,
    value_hash: Option<HashValue>,
}

impl CachedState {
    fn absent() -> Self {
        Self {
            exists: false,
            value_hash: None,
        }
    }

    fn from_state_value(value: Option<StateValue>) -> Self {
        match value {
            Some(state_value) => Self {
                exists: true,
                value_hash: Some(state_value.hash()),
            },
            None => Self::absent(),
        }
    }
}

struct PrefixEffects<'a> {
    state: &'a ChainStateDB,
    cache: HashMap<StateKey, CachedState>,
}

impl<'a> PrefixEffects<'a> {
    fn new(state: &'a ChainStateDB) -> Self {
        Self {
            state,
            cache: HashMap::new(),
        }
    }

    fn get(&mut self, key: &StateKey) -> Option<CachedState> {
        if let Some(entry) = self.cache.get(key) {
            return Some(entry.clone());
        }
        match self.state.get_state_value(key) {
            Ok(opt) => {
                let entry = CachedState::from_state_value(opt);
                self.cache.insert(key.clone(), entry.clone());
                Some(entry)
            }
            Err(err) => {
                warn!(
                    "reuse planner failed to query state for key {:?}: {:?}",
                    key, err
                );
                None
            }
        }
    }

    fn set(&mut self, key: &StateKey, entry: CachedState) {
        self.cache.insert(key.clone(), entry);
    }

    fn apply_write(&mut self, key: &StateKey, op: &WriteOp) {
        use WriteOp::*;
        match op {
            Creation { data, metadata } | Modification { data, metadata } => {
                let state_value = StateValue::new_with_metadata(data.clone(), metadata.clone());
                self.set(
                    key,
                    CachedState {
                        exists: true,
                        value_hash: Some(state_value.hash()),
                    },
                );
            }
            Deletion { .. } => {
                self.set(key, CachedState::absent());
            }
        }
    }
}

impl<'a> ReusePlanner<'a> {
    fn new(
        statedb: &'a ChainStateDB,
        store: Arc<dyn WitnessStore>,
        merge_engine: Arc<MergeEngine>,
        epoch_id: u64,
        meta_fingerprint: Option<HashValue>,
        base_state_root: HashValue,
    ) -> Self {
        Self {
            statedb,
            store,
            merge_engine,
            epoch_id,
            meta_fingerprint,
            base_state_root,
        }
    }

    fn plan(&self, transactions: &[Transaction]) -> Option<ReusePlan> {
        if transactions.is_empty() {
            return Some(ReusePlan {
                entries: Vec::new(),
                stats: PlanStats {
                    total: 0,
                    witness_hits: 0,
                    witness_hits_with_reads: 0,
                    read_checked: 0,
                    plan_time_ms: 0,
                },
            });
        }

        let plan_start = Instant::now();
        let mut entries = Vec::with_capacity(transactions.len());
        let mut witness_hits = 0usize;
        let mut witness_hits_with_reads = 0usize;

        for txn in transactions {
            let (entry, hit, hit_with_reads) = self.prepare_entry(txn);
            if hit {
                witness_hits += 1;
            }
            if hit_with_reads {
                witness_hits_with_reads += 1;
            }
            entries.push(entry);
        }

        let sanitized_execs: Vec<ExecRecord> = entries
            .iter()
            .map(|entry| entry.sanitized.clone())
            .collect();

        let mut prefix = PrefixWrites::default();
        let diff = self
            .merge_engine
            .plan_merge(self.statedb, &mut prefix, &sanitized_execs);
        let decisions = Self::classify_decisions(&sanitized_execs, &diff);

        for (entry, decision) in entries.iter_mut().zip(decisions.into_iter()) {
            entry.decision = decision;
        }

        let mut effects = PrefixEffects::new(self.statedb);
        for entry in entries.iter_mut() {
            match entry.decision {
                PlanDecision::Reuse => {
                    if !entry.reusable || !self.validate_and_apply_reuse(entry, &mut effects) {
                        entry.decision = PlanDecision::Reexec;
                        if entry.forced_reexec {
                            if let Some(witness) = entry.witness.as_ref() {
                                for (key, op) in witness.write_set.iter() {
                                    effects.apply_write(key, op);
                                }
                            }
                        }
                    }
                }
                PlanDecision::Reexec => {
                    if entry.reusable
                        && !entry.forced_reexec
                        && entry.witness.is_some()
                        && self.validate_and_apply_reuse(entry, &mut effects)
                    {
                        entry.decision = PlanDecision::Reuse;
                        continue;
                    }
                    if entry.forced_reexec {
                        if let Some(witness) = entry.witness.as_ref() {
                            for (key, op) in witness.write_set.iter() {
                                effects.apply_write(key, op);
                            }
                        }
                    }
                }
            }
        }

        let elapsed_ms = plan_start.elapsed().as_millis();
        let read_checked = *diff.stats.get("read_checked").unwrap_or(&0);

        Some(ReusePlan {
            entries,
            stats: PlanStats {
                total: transactions.len(),
                witness_hits,
                witness_hits_with_reads,
                read_checked,
                plan_time_ms: elapsed_ms,
            },
        })
    }

    fn prepare_entry(&self, txn: &Transaction) -> (PlanEntry, bool, bool) {
        let key = ExecKey {
            tx_hash: txn.id(),
            epoch_id: self.epoch_id,
        };
        let force_reexec = matches!(
            txn,
            Transaction::BlockMetadata(_) | Transaction::BlockEpilogue(..)
        );
        if let Some(rec) = self.store.get(&key) {
            info!(
                "prepare_entry hit tx={} force_reexec={}",
                key.tx_hash, force_reexec
            );
            let mut sanitized = rec.clone();
            let meta_ok = match (rec.meta_fingerprint, self.meta_fingerprint) {
                (Some(a), Some(b)) => a == b,
                (Some(_), None) => false,
                _ => true,
            };
            let base_ok = rec
                .base_state_root
                .map(|root| root == self.base_state_root)
                .unwrap_or(false);
            let supported =
                !force_reexec && meta_ok && base_ok && Self::record_supported_for_reuse(&rec);
            let mut hit_with_reads = false;

            if !supported {
                sanitized.read_set = None;
            } else if sanitized.read_set.is_some() {
                hit_with_reads = true;
            }
            (
                PlanEntry {
                    sanitized,
                    witness: Some(rec),
                    decision: PlanDecision::Reexec,
                    forced_reexec: force_reexec,
                    reusable: supported,
                },
                true,
                hit_with_reads,
            )
        } else {
            (
                PlanEntry {
                    sanitized: Self::placeholder_exec_record(txn.id(), self.epoch_id),
                    witness: None,
                    decision: PlanDecision::Reexec,
                    forced_reexec: false,
                    reusable: false,
                },
                false,
                false,
            )
        }
    }

    fn record_supported_for_reuse(rec: &ExecRecord) -> bool {
        if rec.base_state_root.is_none() {
            return false;
        }
        if !rec.table_infos.is_empty() {
            return false;
        }
        if rec
            .write_set
            .iter()
            .any(|(key, _)| !Self::is_supported_write_key(key))
        {
            return false;
        }
        match rec.read_set.as_ref() {
            Some(reads) if !reads.is_empty() => {
                if reads
                    .iter()
                    .any(|entry| !Self::is_supported_read_key(&entry.key))
                {
                    return false;
                }
                true
            }
            _ => false,
        }
    }

    fn classify_decisions(sanitized: &[ExecRecord], diff: &MergeDiff) -> Vec<PlanDecision> {
        let reused: HashSet<_> = diff.reused.iter().cloned().collect();
        sanitized
            .iter()
            .map(|rec| {
                let decision = if reused.contains(&rec.tx_hash) {
                    PlanDecision::Reuse
                } else {
                    PlanDecision::Reexec
                };
                info!(
                    "classify_decision tx={} decision={:?}",
                    rec.tx_hash, decision
                );
                decision
            })
            .collect()
    }

    fn placeholder_exec_record(tx_hash: HashValue, epoch_id: u64) -> ExecRecord {
        ExecRecord {
            tx_hash,
            epoch_id,
            base_state_root: None,
            read_set: None,
            write_set: Vec::new(),
            event_root: HashValue::zero(),
            gas: 0,
            status_ok: false,
            meta_fingerprint: None,
            status: None,
            events: Vec::new(),
            table_infos: Vec::new(),
        }
    }

    fn is_supported_write_key(key: &StateKey) -> bool {
        matches!(key.inner(), StateKeyInner::AccessPath(_))
    }

    fn is_supported_read_key(key: &StateKey) -> bool {
        matches!(
            key.inner(),
            StateKeyInner::AccessPath(_) | StateKeyInner::TableItem { .. }
        )
    }

    fn validate_and_apply_reuse(&self, entry: &PlanEntry, effects: &mut PrefixEffects) -> bool {
        use starcoin_vm2_vm_types::write_set::WriteOp::{Creation, Deletion, Modification};

        let rec = match entry.witness.as_ref() {
            Some(rec) => rec,
            None => return false,
        };

        let read_map = match rec.read_set.as_ref() {
            Some(reads) => {
                let mut map = HashMap::with_capacity(reads.len());
                for read in reads {
                    map.entry(read.key.clone())
                        .or_insert((read.existed, read.value_hash));
                }
                map
            }
            None => return false,
        };
        if read_map.is_empty() {
            info!("[VM2 reuse] missing reads for tx={} – fallback reexec", rec.tx_hash);
            return false;
        }

        for (key, _) in rec.write_set.iter() {
            if !read_map.contains_key(key) {
                info!(
                    "[VM2 reuse] missing write coverage tx={} key={:?}",
                    rec.tx_hash, key
                );
                return false;
            }
        }

        // Verify that every recorded read matches the current (or prefixed) state.
        for (key, (expected_exists, expected_hash)) in read_map.iter() {
            let entry = match effects.get(key) {
                Some(entry) => entry,
                None => {
                    info!(
                        "[VM2 reuse] lookup miss tx={} key={:?}",
                        rec.tx_hash, key
                    );
                    return false;
                }
            };

            if entry.exists != *expected_exists {
                info!(
                    "[VM2 reuse] exists mismatch tx={} key={:?} expected={} actual={}",
                    rec.tx_hash, key, expected_exists, entry.exists
                );
                return false;
            }

            if *expected_exists && entry.value_hash.as_ref() != Some(expected_hash) {
                info!(
                    "[VM2 reuse] hash mismatch tx={} key={:?} expected={:?} actual={:?}",
                    rec.tx_hash, key, expected_hash, entry.value_hash
                );
                return false;
            }
        }

        for (key, op) in rec.write_set.iter() {
            match op {
                Creation { .. } => {
                    let before = match effects.get(key) {
                        Some(entry) => entry,
                        None => {
                            info!(
                                "[VM2 reuse] creation lookup miss tx={} key={:?}",
                                rec.tx_hash, key
                            );
                            return false;
                        }
                    };
                    if before.exists {
                        info!(
                            "[VM2 reuse] creation expected absent tx={} key={:?}",
                            rec.tx_hash, key
                        );
                        return false;
                    }
                    if let Some((expected_exists, _)) = read_map.get(key) {
                        if *expected_exists {
                            info!(
                                "[VM2 reuse] creation read-set mismatch tx={} key={:?}",
                                rec.tx_hash, key
                            );
                            return false;
                        }
                    }
                    effects.apply_write(key, op);
                }
                Modification { .. } => {
                    let before = match effects.get(key) {
                        Some(entry) => entry,
                        None => {
                            info!(
                                "[VM2 reuse] modify lookup miss tx={} key={:?}",
                                rec.tx_hash, key
                            );
                            return false;
                        }
                    };
                    let (expected_exists, expected_hash) = match read_map.get(key) {
                        Some((exists, hash)) => (*exists, hash),
                        None => {
                            info!(
                                "[VM2 reuse] modify missing read-set entry tx={} key={:?}",
                                rec.tx_hash, key
                            );
                            return false;
                        }
                    };
                    if !expected_exists || !before.exists {
                        info!(
                            "[VM2 reuse] modify exists mismatch tx={} key={:?} expected_exists={} before.exists={}",
                            rec.tx_hash, key, expected_exists, before.exists
                        );
                        return false;
                    }
                    if before.value_hash.as_ref() != Some(expected_hash) {
                        info!(
                            "[VM2 reuse] modify hash mismatch tx={} key={:?} expected={:?} actual={:?}",
                            rec.tx_hash, key, expected_hash, before.value_hash
                        );
                        return false;
                    }
                    effects.apply_write(key, op);
                }
                Deletion { .. } => {
                    let before = match effects.get(key) {
                        Some(entry) => entry,
                        None => {
                            info!(
                                "reuse_delete_lookup_miss tx={} key={:?}",
                                rec.tx_hash, key
                            );
                            return false;
                        }
                    };
                    let (expected_exists, expected_hash) = match read_map.get(key) {
                        Some((exists, hash)) => (*exists, hash),
                        None => {
                            info!(
                                "[VM2 reuse] delete missing read-set entry tx={} key={:?}",
                                rec.tx_hash, key
                            );
                            return false;
                        }
                    };
                    if !expected_exists || !before.exists {
                        info!(
                            "[VM2 reuse] delete exists mismatch tx={} key={:?} expected_exists={} before.exists={}",
                            rec.tx_hash, key, expected_exists, before.exists
                        );
                        return false;
                    }
                    if before.value_hash.as_ref() != Some(expected_hash) {
                        info!(
                            "[VM2 reuse] delete hash mismatch tx={} key={:?} expected={:?} actual={:?}",
                            rec.tx_hash, key, expected_hash, before.value_hash
                        );
                        return false;
                    }
                    effects.apply_write(key, op);
                }
            }
        }

        info!("reuse_validate_success tx={}", rec.tx_hash);
        true
    }
}

struct ReuseExecutor<'a> {
    statedb: &'a ChainStateDB,
    store: Arc<dyn WitnessStore>,
    epoch_id: u64,
    meta_fingerprint: Option<HashValue>,
    base_state_root: HashValue,
}

impl<'a> ReuseExecutor<'a> {
    fn new(
        statedb: &'a ChainStateDB,
        epoch_id: u64,
        store: Arc<dyn WitnessStore>,
        meta_fingerprint: Option<HashValue>,
        base_state_root: HashValue,
    ) -> Self {
        Self {
            statedb,
            store,
            epoch_id,
            meta_fingerprint,
            base_state_root,
        }
    }

    fn execute_plan(
        &self,
        transactions: &[Transaction],
        gas_limit: u64,
        vm_metrics: Option<VMMetrics>,
        plan: ReusePlan,
    ) -> ExecutorResult<BlockExecutedData> {
        debug_assert_eq!(transactions.len(), plan.len());

        let entries = plan.entries;
        let total = entries.len();
        let mut txn_infos = Vec::with_capacity(total);
        let mut txn_events: Vec<Vec<ContractEvent>> = Vec::with_capacity(total);
        let mut write_sets: Vec<WriteSet> = Vec::with_capacity(total);
        let mut remaining_gas = gas_limit;
        let last_index = total.saturating_sub(1);
        let mut pending_reuse_writes = WriteSetMut::default();

        for (idx, txn) in transactions.iter().enumerate() {
            let entry = &entries[idx];
            let tx_hash = txn.id();
            let should_commit = total > 0 && (idx == 0 || idx == last_index);
            let fee_payer = match txn {
                Transaction::UserTransaction(user_txn) => Some(user_txn.sender()),
                _ => None,
            };

            match entry.decision {
                PlanDecision::Reuse => {
                    if let Some(rec) = entry.witness.as_ref() {
                        if rec.base_state_root != Some(self.base_state_root) {
                            info!(
                                "reuse_base_mismatch tx={} expected={:?} actual={:?}",
                                tx_hash,
                                self.base_state_root,
                                rec.base_state_root
                            );
                            self.remove_witness(tx_hash);
                        } else if let Some(status) = rec.status.clone() {
                            if matches!(status, KeptVMStatus::Executed) && rec.status_ok {
                                let write_entries = rec.write_set.clone();
                                let write_set_mut = WriteSetMut::new(write_entries.clone());
                                let ws_for_result = write_set_mut
                                    .freeze()
                                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                                info!(
                                    "reuse_plan_execute apply witness tx={} writes={}",
                                    tx_hash,
                                    write_entries.len()
                                );
                                for (idx, (key, op)) in write_entries.iter().enumerate() {
                                    info!(
                                        "reuse_plan_write tx={} idx={} key={:?} op={:?}",
                                        tx_hash, idx, key, op
                                    );
                                }

                                pending_reuse_writes.extend(write_entries);
                                if let Some(payer) = fee_payer {
                                    record_fee_payer_for_reuse(payer);
                                }

                                let next_decision = entries
                                    .get(idx + 1)
                                    .map(|next| next.decision)
                                    .unwrap_or(PlanDecision::Reexec);
                                let need_apply_before_next = next_decision == PlanDecision::Reexec;

                                if need_apply_before_next || should_commit {
                                    self.apply_pending_reuse_writes(&mut pending_reuse_writes)?;
                                }

                                let txn_state_root = if should_commit {
                                    Some(
                                        self.statedb
                                            .commit()
                                            .map_err(BlockExecutorError::BlockChainStateErr)?,
                                    )
                                } else {
                                    None
                                };

                                let events = rec.events.clone();
                                let txn_info = TransactionInfo::new(
                                    tx_hash,
                                    txn_state_root,
                                    events.as_slice(),
                                    rec.gas,
                                    status.clone(),
                                );

                                txn_infos.push(txn_info);
                                txn_events.push(events);
                                write_sets.push(ws_for_result);
                                remaining_gas = remaining_gas.saturating_sub(rec.gas);

                                // Refresh witness to keep it hot in the cache.
                                self.store.put(rec.clone());
                                continue;
                            }
                        }
                    }
                    // Witness missing or unusable: fall through to re-exec.
                }
                PlanDecision::Reexec => {
                    // handled below
                }
            }

            if !pending_reuse_writes.is_empty() {
                self.apply_pending_reuse_writes(&mut pending_reuse_writes)?;
            }

            reuse_recorder::start();
            let vm_outputs = do_execute_block_transactions(
                self.statedb,
                vec![txn.clone()],
                Some(remaining_gas),
                vm_metrics.clone(),
            )
            .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
            let recorded_reads = reuse_recorder::finish();

            let output = vm_outputs
                .into_iter()
                .next()
                .ok_or(BlockExecutorError::BlockTransactionZero)?;
            let mut read_sets_iter = recorded_reads.into_iter();

            let (write_set, events, gas_used, status, _aux) = output.into_inner();
            match status {
                TransactionStatus::Discard(discard_status) => {
                    info!(
                        "reuse reexec discard tx={} status={:?}",
                        tx_hash, discard_status
                    );
                    self.remove_witness(tx_hash);
                    return Err(BlockExecutorError::BlockTransactionDiscard(
                        discard_status,
                        tx_hash,
                    ));
                }
                TransactionStatus::Retry => {
                    info!("reuse reexec retry tx={}", tx_hash);
                    self.remove_witness(tx_hash);
                    return Err(BlockExecutorError::BlockExecuteRetryErr);
                }
                TransactionStatus::Keep(keep_status) => {
                    info!("reuse reexec keep tx={} status={:?}", tx_hash, keep_status);
                    let write_set_clone = write_set.clone();
                    let write_entries_for_store =
                        write_set_clone.clone().into_iter().collect::<Vec<_>>();
                    info!(
                        "reuse_plan_reexec tx={} write_entries={}",
                        tx_hash,
                        write_entries_for_store.len()
                    );
                    let mut read_entries = read_sets_iter.next().map(Self::to_read_entries);
                    crate::hydrate_read_set_for_writes(
                        self.statedb,
                        &mut read_entries,
                        &write_entries_for_store,
                    )
                    .map_err(|e| BlockExecutorError::BlockChainStateErr(e.into()))?;

                    self.statedb
                        .apply_write_set(write_set)
                        .map_err(BlockExecutorError::BlockChainStateErr)?;
                    let txn_state_root = if should_commit {
                        Some(
                            self.statedb
                                .commit()
                                .map_err(BlockExecutorError::BlockChainStateErr)?,
                        )
                    } else {
                        None
                    };

                    let txn_info = TransactionInfo::new(
                        tx_hash,
                        txn_state_root,
                        events.as_slice(),
                        gas_used,
                        keep_status.clone(),
                    );
                    let event_root = txn_info.event_root_hash();
                    let events_for_result = events.clone();

                    txn_infos.push(txn_info);
                    txn_events.push(events_for_result);
                    write_sets.push(write_set_clone);
                    remaining_gas = remaining_gas.saturating_sub(gas_used);

                    self.store_exec_record(
                        tx_hash,
                        gas_used,
                        event_root,
                        keep_status,
                        read_entries,
                        write_entries_for_store,
                        events,
                        self.txn_meta_fingerprint(txn),
                    );
                }
            }
        }

        if !pending_reuse_writes.is_empty() {
            self.apply_pending_reuse_writes(&mut pending_reuse_writes)?;
        }

        let final_root = self.statedb.state_root();
        info!("reuse_execute_plan final_root={:?}", final_root);
        Ok(BlockExecutedData {
            state_root: final_root,
            txn_infos,
            txn_events,
            txn_table_infos: Default::default(),
            write_sets,
        })
    }

    fn execute_full(
        &self,
        transactions: Vec<Transaction>,
        gas_limit: u64,
        vm_metrics: Option<VMMetrics>,
        record_reads: bool,
    ) -> ExecutorResult<BlockExecutedData> {
        let (executed, recorded_reads) = if record_reads {
            reuse_recorder::start();
            let exec_res = block_executor::block_execute(
                self.statedb,
                transactions.clone(),
                gas_limit,
                vm_metrics,
            );
            match exec_res {
                Ok(data) => (data, reuse_recorder::finish()),
                Err(err) => {
                    reuse_recorder::finish();
                    return Err(err);
                }
            }
        } else {
            (
                block_executor::block_execute(
                    self.statedb,
                    transactions.clone(),
                    gas_limit,
                    vm_metrics,
                )?,
                Vec::<Vec<reuse_recorder::ReadDescriptor>>::new(),
            )
        };

        for (idx, (txn, info)) in transactions
            .iter()
            .take(executed.txn_infos.len())
            .zip(executed.txn_infos.iter())
            .enumerate()
        {
            let write_set_entries = executed
                .write_sets
                .get(idx)
                .map(|ws| ws.clone().into_iter().collect::<Vec<_>>())
                .unwrap_or_default();
            let read_set_entries = recorded_reads.get(idx).cloned().map(Self::to_read_entries);
            let events = executed.txn_events.get(idx).cloned().unwrap_or_default();
            let status_clone = info.status().clone();

            self.store_exec_record(
                txn.id(),
                info.gas_used(),
                info.event_root_hash(),
                status_clone,
                read_set_entries,
                write_set_entries,
                events,
                self.txn_meta_fingerprint(txn),
            );
        }

        TEST_REUSE_REEXEC.fetch_add(executed.txn_infos.len(), Ordering::Relaxed);
        info!("reuse_execute_full final_root={:?}", executed.state_root);
        Ok(executed)
    }

    fn txn_meta_fingerprint(&self, txn: &Transaction) -> Option<HashValue> {
        match txn {
            Transaction::UserTransaction(_) | Transaction::BlockEpilogue(..) => {
                self.meta_fingerprint
            }
            _ => None,
        }
    }

    fn apply_pending_reuse_writes(
        &self,
        pending: &mut WriteSetMut,
    ) -> Result<(), BlockExecutorError> {
        if pending.is_empty() {
            return Ok(());
        }

        let mut to_apply = WriteSetMut::default();
        std::mem::swap(&mut to_apply, pending);
        let frozen = to_apply
            .freeze()
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        self.statedb
            .apply_write_set(frozen)
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        Ok(())
    }

    fn remove_witness(&self, tx_hash: HashValue) {
        let key = ExecKey {
            tx_hash,
            epoch_id: self.epoch_id,
        };
        self.store.remove(&key);
    }

    fn store_exec_record(
        &self,
        tx_hash: HashValue,
        gas_used: u64,
        event_root: HashValue,
        status: KeptVMStatus,
        read_set: Option<Vec<crate::ReadEntry>>,
        write_set: Vec<(StateKey, WriteOp)>,
        events: Vec<ContractEvent>,
        meta_fingerprint: Option<HashValue>,
    ) {
        let rec = ExecRecord {
            tx_hash,
            epoch_id: self.epoch_id,
            base_state_root: Some(self.base_state_root),
            read_set,
            write_set,
            event_root,
            gas: gas_used,
            status_ok: matches!(status, KeptVMStatus::Executed),
            meta_fingerprint,
            status: Some(status),
            events,
            table_infos: Vec::new(),
        };
        self.store.put(rec);
    }

    fn to_read_entries(reads: Vec<reuse_recorder::ReadDescriptor>) -> Vec<crate::ReadEntry> {
        reads
            .into_iter()
            .map(|desc| crate::ReadEntry {
                key: desc.key,
                from_storage: desc.from_storage,
                existed: desc.existed,
                value_hash: desc.value_hash,
            })
            .collect()
    }
}
