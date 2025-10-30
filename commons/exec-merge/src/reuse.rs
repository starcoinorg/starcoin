use crate::{ExecKey, ExecRecord, MergeDiff, MergeEngine, PrefixWrites, ReuseOpts, WitnessStore};
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
use starcoin_vm2_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};
use starcoin_vm2_vm_types::state_store::TStateView;
use starcoin_vm_runtime::reuse_recorder;
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
    let executor = ReuseExecutor::new(statedb, opts.epoch_id, opts.witness_store.clone());

    if opts.enabled {
        let planner = ReusePlanner::new(
            statedb,
            opts.witness_store.clone(),
            opts.merge_engine.clone(),
            opts.epoch_id,
        );
        if let Some(plan) = planner.plan(&transactions) {
            let reused = plan.reused_count();
            let reexec = plan.reexec_count();
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

struct ReusePlanner<'a> {
    statedb: &'a ChainStateDB,
    store: Arc<dyn WitnessStore>,
    merge_engine: Arc<MergeEngine>,
    epoch_id: u64,
}

struct PrefixEffects<'a> {
    state: &'a ChainStateDB,
    cache: HashMap<StateKey, bool>,
}

impl<'a> PrefixEffects<'a> {
    fn new(state: &'a ChainStateDB) -> Self {
        Self {
            state,
            cache: HashMap::new(),
        }
    }

    fn exists(&mut self, key: &StateKey) -> Option<bool> {
        if let Some(exists) = self.cache.get(key) {
            return Some(*exists);
        }
        match self.state.get_state_value(key) {
            Ok(opt) => {
                let exists = opt.is_some();
                self.cache.insert(key.clone(), exists);
                Some(exists)
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

    fn set(&mut self, key: &StateKey, exists: bool) {
        self.cache.insert(key.clone(), exists);
    }
}

impl<'a> ReusePlanner<'a> {
    fn new(
        statedb: &'a ChainStateDB,
        store: Arc<dyn WitnessStore>,
        merge_engine: Arc<MergeEngine>,
        epoch_id: u64,
    ) -> Self {
        Self {
            statedb,
            store,
            merge_engine,
            epoch_id,
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
            if entry.decision == PlanDecision::Reuse
                && !self.validate_and_apply_reuse(entry, &mut effects)
            {
                entry.decision = PlanDecision::Reexec;
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
        if let Some(rec) = self.store.get(&key) {
            let mut sanitized = rec.clone();
            let supported = Self::record_supported_for_reuse(&rec);
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
                },
                false,
                false,
            )
        }
    }

    fn record_supported_for_reuse(rec: &ExecRecord) -> bool {
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
        if let Some(reads) = rec.read_set.as_ref() {
            if reads
                .iter()
                .any(|entry| !Self::is_supported_read_key(&entry.key))
            {
                return false;
            }
        }
        true
    }

    fn classify_decisions(sanitized: &[ExecRecord], diff: &MergeDiff) -> Vec<PlanDecision> {
        let reused: HashSet<_> = diff.reused.iter().cloned().collect();
        sanitized
            .iter()
            .map(|rec| {
                if reused.contains(&rec.tx_hash) {
                    PlanDecision::Reuse
                } else {
                    PlanDecision::Reexec
                }
            })
            .collect()
    }

    fn placeholder_exec_record(tx_hash: HashValue, epoch_id: u64) -> ExecRecord {
        ExecRecord {
            tx_hash,
            epoch_id,
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

    fn validate_and_apply_reuse(
        &self,
        entry: &PlanEntry,
        effects: &mut PrefixEffects,
    ) -> bool {
        use starcoin_vm2_vm_types::write_set::WriteOp::{Creation, Deletion, Modification};

        let rec = match entry.witness.as_ref() {
            Some(rec) => rec,
            None => return false,
        };

        let read_map = match rec.read_set.as_ref() {
            Some(reads) => {
                let mut map = HashMap::with_capacity(reads.len());
                for read in reads {
                    map.insert(read.key.clone(), read.existed);
                }
                map
            }
            None => return false,
        };

        for (key, op) in rec.write_set.iter() {
            match op {
                Creation { .. } => {
                    if read_map.get(key).copied() != Some(false) {
                        return false;
                    }
                    match effects.exists(key) {
                        Some(false) => effects.set(key, true),
                        Some(true) => return false,
                        None => return false,
                    }
                }
                Modification { .. } => {
                    if read_map.get(key).copied() != Some(true) {
                        return false;
                    }
                    match effects.exists(key) {
                        Some(true) => effects.set(key, true),
                        Some(false) => return false,
                        None => return false,
                    }
                }
                Deletion { .. } => {
                    if read_map.get(key).copied() != Some(true) {
                        return false;
                    }
                    match effects.exists(key) {
                        Some(true) => effects.set(key, false),
                        Some(false) => return false,
                        None => return false,
                    }
                }
            }
        }

        true
    }
}

struct ReuseExecutor<'a> {
    statedb: &'a ChainStateDB,
    store: Arc<dyn WitnessStore>,
    epoch_id: u64,
}

impl<'a> ReuseExecutor<'a> {
    fn new(statedb: &'a ChainStateDB, epoch_id: u64, store: Arc<dyn WitnessStore>) -> Self {
        Self {
            statedb,
            store,
            epoch_id,
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

            match entry.decision {
                PlanDecision::Reuse => {
                    if let Some(rec) = entry.witness.as_ref() {
                        if let Some(status) = rec.status.clone() {
                            if matches!(status, KeptVMStatus::Executed) && rec.status_ok {
                                let write_entries = rec.write_set.clone();
                                let write_set_mut = WriteSetMut::new(write_entries.clone());
                                let ws_for_result = write_set_mut
                                    .freeze()
                                    .map_err(BlockExecutorError::BlockChainStateErr)?;

                                pending_reuse_writes.extend(write_entries);

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
                    return Err(BlockExecutorError::BlockTransactionDiscard(
                        discard_status,
                        tx_hash,
                    ));
                }
                TransactionStatus::Retry => {
                    return Err(BlockExecutorError::BlockExecuteRetryErr);
                }
                TransactionStatus::Keep(keep_status) => {
                    let read_entries = read_sets_iter.next().map(Self::to_read_entries);
                    let write_set_clone = write_set.clone();
                    let write_entries_for_store =
                        write_set_clone.clone().into_iter().collect::<Vec<_>>();

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
                    );
                }
            }
        }

        if !pending_reuse_writes.is_empty() {
            self.apply_pending_reuse_writes(&mut pending_reuse_writes)?;
        }

        Ok(BlockExecutedData {
            state_root: self.statedb.state_root(),
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
            );
        }

        TEST_REUSE_REEXEC.fetch_add(executed.txn_infos.len(), Ordering::Relaxed);
        Ok(executed)
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

    fn store_exec_record(
        &self,
        tx_hash: HashValue,
        gas_used: u64,
        event_root: HashValue,
        status: KeptVMStatus,
        read_set: Option<Vec<crate::ReadEntry>>,
        write_set: Vec<(StateKey, WriteOp)>,
        events: Vec<ContractEvent>,
    ) {
        let rec = ExecRecord {
            tx_hash,
            epoch_id: self.epoch_id,
            read_set,
            write_set,
            event_root,
            gas: gas_used,
            status_ok: matches!(status, KeptVMStatus::Executed),
            meta_fingerprint: None,
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
