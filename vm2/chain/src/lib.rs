// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_logger::prelude::*;
use starcoin_vm2_crypto::hash::{CryptoHash, PlainCryptoHash};
use starcoin_vm2_executor::{
    block_executor::{self, BlockExecutedData, VMMetrics},
    do_execute_block_transactions,
};
use starcoin_vm2_state_api::{AccountStateReader, ChainStateReader, ChainStateWriter};
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::block_metadata::BlockMetadata;
use starcoin_vm2_types::block_metadata::BlockMetadata as BlockMetadata2;
use starcoin_vm2_types::contract_event::ContractEvent;
use starcoin_vm2_types::error::{BlockExecutorError, ExecutorResult};
use starcoin_vm2_types::transaction::{
    SignedUserTransaction, Transaction, TransactionInfo, TransactionStatus,
};
use starcoin_vm2_types::vm_error::KeptVMStatus;
use starcoin_vm2_vm_types::account_config::genesis_address;
use starcoin_vm2_vm_types::on_chain_resource::Epoch;
use starcoin_vm2_vm_types::state_store::{
    state_key::{inner::StateKeyInner, StateKey},
    TStateView,
};
use starcoin_vm2_vm_types::write_set::{WriteSet, WriteSetMut};
// reuse imports
use starcoin_exec_merge as exec_merge;
use starcoin_exec_merge::{ExecKey, ExecRecord, ReuseOpts, StateViewExt};
use starcoin_vm_runtime::reuse_recorder;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

struct ChainStateValueView<'a> {
    state: &'a ChainStateDB,
}

impl<'a> StateViewExt for ChainStateValueView<'a> {
    fn get_value_hash(
        &self,
        key: &starcoin_vm2_vm_types::state_store::state_key::StateKey,
    ) -> Option<starcoin_vm2_crypto::HashValue> {
        self.state
            .get_state_value(key)
            .ok()
            .and_then(|maybe_value| maybe_value.map(|state_value| state_value.hash()))
    }
}

#[derive(Debug)]
struct PlanStats {
    total: usize,
    witness_hits: usize,
    witness_hits_with_reads: usize,
    witness_missing: usize,
    reused: usize,
    reexec: usize,
    read_checked: u64,
    plan_time_ms: u128,
}

#[derive(Debug)]
struct PlanOutcome {
    stats: PlanStats,
    reused_indices: Vec<usize>,
    reexec_indices: Vec<usize>,
    records: Vec<Option<ExecRecord>>,
}

static TEST_REUSE_HITS: AtomicUsize = AtomicUsize::new(0);
static TEST_REUSE_REEXEC: AtomicUsize = AtomicUsize::new(0);

fn is_supported_state_key(key: &StateKey) -> bool {
    matches!(key.inner(), StateKeyInner::AccessPath(_))
}

fn record_supported_for_reuse(rec: &ExecRecord) -> bool {
    if !rec.table_infos.is_empty() {
        return false;
    }
    if rec
        .write_set
        .iter()
        .any(|(key, _)| !is_supported_state_key(key))
    {
        return false;
    }
    if let Some(reads) = rec.read_set.as_ref() {
        if reads
            .iter()
            .any(|entry| !is_supported_state_key(&entry.key))
        {
            return false;
        }
    }
    true
}

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

pub fn execute_transactions(
    statedb: &ChainStateDB,
    transactions: Vec<Transaction>,
    gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
) -> ExecutorResult<BlockExecutedData> {
    // This function will execute the transactions in the block using vm2
    // Note: The actual implementation of VM2 execution and saving logic will depend on your VM2 setup.
    let executed_data =
        block_executor::block_execute(statedb, transactions, gas_limit, vm_metrics)?;

    Ok(executed_data)
}

/// Execute with reuse fast-path (skeleton). When `opts.enabled` is false, falls back to execute_transactions.
fn to_read_entries(reads: Vec<reuse_recorder::ReadDescriptor>) -> Vec<exec_merge::ReadEntry> {
    reads
        .into_iter()
        .map(|desc| exec_merge::ReadEntry {
            key: desc.key,
            from_storage: desc.from_storage,
            existed: desc.existed,
            value_hash: desc.value_hash,
        })
        .collect()
}

pub fn execute_transactions_with_reuse(
    statedb: &ChainStateDB,
    transactions: Vec<Transaction>,
    gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
    opts: ReuseOpts,
) -> ExecutorResult<BlockExecutedData> {
    let pre_fp = opts.pre_state_fingerprint;
    let store = opts.witness_store.clone();
    let merge_engine = opts.merge_engine.clone();

    let plan_outcome = if opts.enabled {
        let plan_start = Instant::now();
        let mut plan_execs = Vec::with_capacity(transactions.len());
        let mut planned_records = Vec::with_capacity(transactions.len());
        let mut witness_hits = 0usize;
        let mut witness_hits_with_reads = 0usize;

        for tx in transactions.iter() {
            let key = ExecKey {
                tx_hash: tx.id(),
                pre_state_fingerprint: pre_fp,
            };
            if let Some(mut rec) = store.get(&key) {
                if !record_supported_for_reuse(&rec) {
                    rec.read_set = None;
                }
                if rec.read_set.is_some() {
                    witness_hits_with_reads += 1;
                }
                witness_hits += 1;
                plan_execs.push(rec.clone());
                planned_records.push(Some(rec));
            } else {
                plan_execs.push(ExecRecord {
                    tx_hash: tx.id(),
                    pre_state_fingerprint: pre_fp,
                    read_set: None,
                    write_set: Vec::new(),
                    event_root: starcoin_vm2_crypto::HashValue::zero(),
                    gas: 0,
                    status_ok: false,
                    meta_fingerprint: None,
                    status: None,
                    events: Vec::new(),
                    table_infos: Vec::new(),
                });
                planned_records.push(None);
            }
        }

        let mut prefix = exec_merge::PrefixWrites::default();
        let diff = merge_engine.plan_merge(
            &ChainStateValueView { state: statedb },
            &mut prefix,
            &plan_execs,
        );
        let mut reused_indices = Vec::with_capacity(diff.reused.len());
        let mut reexec_indices = Vec::with_capacity(diff.reexec.len());
        if !diff.reused.is_empty() || !diff.reexec.is_empty() {
            let mut reused_hashes = diff.reused.iter();
            let mut reexec_hashes = diff.reexec.iter();
            let mut next_reused = reused_hashes.next();
            let mut next_reexec = reexec_hashes.next();
            for (idx, rec) in plan_execs.iter().enumerate() {
                let tx_hash = &rec.tx_hash;
                if let Some(expected) = next_reused {
                    if tx_hash == expected {
                        reused_indices.push(idx);
                        next_reused = reused_hashes.next();
                        continue;
                    }
                }
                if let Some(expected) = next_reexec {
                    if tx_hash == expected {
                        reexec_indices.push(idx);
                        next_reexec = reexec_hashes.next();
                    }
                }
                if next_reused.is_none() && next_reexec.is_none() {
                    break;
                }
            }
        }
        let elapsed_ms = plan_start.elapsed().as_millis();
        let read_checked = *diff.stats.get("read_checked").unwrap_or(&0);
        let reused_count = diff.reused.len();
        let reexec_count = diff.reexec.len();
        Some(PlanOutcome {
            stats: PlanStats {
                total: plan_execs.len(),
                witness_hits,
                witness_hits_with_reads,
                witness_missing: plan_execs.len().saturating_sub(witness_hits),
                reused: reused_count,
                reexec: reexec_count,
                read_checked,
                plan_time_ms: elapsed_ms,
            },
            reused_indices,
            reexec_indices,
            records: planned_records,
        })
    } else {
        None
    };

    if let Some(plan) = plan_outcome {
        TEST_REUSE_HITS.fetch_add(plan.stats.reused, Ordering::Relaxed);
        TEST_REUSE_REEXEC.fetch_add(plan.stats.reexec, Ordering::Relaxed);
        let result = execute_with_plan(
            statedb,
            &transactions,
            gas_limit,
            vm_metrics.clone(),
            pre_fp,
            store.clone(),
            &plan,
        )?;
        if plan.stats.reexec != plan.reexec_indices.len() {
            warn!(
                "vm2 reuse plan reexec count mismatch: stats={}, indices={}",
                plan.stats.reexec,
                plan.reexec_indices.len()
            );
        }
        if plan.stats.reused != plan.reused_indices.len() {
            warn!(
                "vm2 reuse plan reuse count mismatch: stats={}, indices={}",
                plan.stats.reused,
                plan.reused_indices.len()
            );
        }
        let stats = &plan.stats;
        debug!(
            "vm2 reuse plan summary: total={}, witness_hits={}, witness_hits_with_reads={}, reused={}, reexec={}, missing={}, read_checked={}, plan_time_ms={}",
            stats.total,
            stats.witness_hits,
            stats.witness_hits_with_reads,
            stats.reused,
            stats.reexec,
            stats.witness_missing,
            stats.read_checked,
            stats.plan_time_ms,
        );
        return Ok(result);
    }

    execute_full_execution(
        statedb,
        transactions,
        gas_limit,
        vm_metrics,
        opts.enabled,
        pre_fp,
        store,
    )
}

fn execute_full_execution(
    statedb: &ChainStateDB,
    transactions: Vec<Transaction>,
    gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
    recording: bool,
    pre_fp: starcoin_vm2_crypto::HashValue,
    store: Arc<dyn exec_merge::WitnessStore>,
) -> ExecutorResult<BlockExecutedData> {
    let (executed, recorded_reads) = if recording {
        reuse_recorder::start();
        let exec_res =
            block_executor::block_execute(statedb, transactions.clone(), gas_limit, vm_metrics);
        match exec_res {
            Ok(data) => (data, reuse_recorder::finish()),
            Err(err) => {
                reuse_recorder::finish();
                return Err(err);
            }
        }
    } else {
        (
            block_executor::block_execute(statedb, transactions.clone(), gas_limit, vm_metrics)?,
            Vec::<Vec<reuse_recorder::ReadDescriptor>>::new(),
        )
    };

    for (idx, (tx, info)) in transactions
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
        let read_set_entries = recorded_reads.get(idx).cloned().map(to_read_entries);
        let events = executed.txn_events.get(idx).cloned().unwrap_or_default();
        let status_clone = info.status().clone();

        let rec = ExecRecord {
            tx_hash: tx.id(),
            pre_state_fingerprint: pre_fp,
            read_set: read_set_entries,
            write_set: write_set_entries,
            event_root: info.event_root_hash(),
            gas: info.gas_used(),
            status_ok: matches!(info.status(), KeptVMStatus::Executed),
            meta_fingerprint: None,
            status: Some(status_clone),
            events,
            table_infos: Vec::new(),
        };
        store.put(rec);
    }

    TEST_REUSE_REEXEC.fetch_add(executed.txn_infos.len(), Ordering::Relaxed);

    Ok(executed)
}

fn apply_pending_reuse_writes(
    statedb: &ChainStateDB,
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
    statedb
        .apply_write_set(frozen)
        .map_err(BlockExecutorError::BlockChainStateErr)?;
    Ok(())
}

fn execute_with_plan(
    statedb: &ChainStateDB,
    transactions: &[Transaction],
    gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
    pre_fp: starcoin_vm2_crypto::HashValue,
    store: Arc<dyn exec_merge::WitnessStore>,
    plan: &PlanOutcome,
) -> ExecutorResult<BlockExecutedData> {
    let mut txn_infos = Vec::with_capacity(transactions.len());
    let mut txn_events: Vec<Vec<ContractEvent>> = Vec::with_capacity(transactions.len());
    let mut write_sets: Vec<WriteSet> = Vec::with_capacity(transactions.len());
    let mut remaining_gas = gas_limit;
    let last_index = transactions.len().saturating_sub(1);
    let reused_indices: HashSet<usize> = plan.reused_indices.iter().copied().collect();
    let mut pending_reuse_writes = WriteSetMut::default();

    for (idx, txn) in transactions.iter().enumerate() {
        let tx_hash = txn.id();
        let should_commit = !transactions.is_empty() && (idx == 0 || idx == last_index);
        let mut reused = false;

        if reused_indices.contains(&idx) {
            if let Some(rec) = plan.records.get(idx).and_then(|r| r.as_ref()) {
                if rec.status_ok {
                    if let Some(status) = rec.status.clone() {
                        if matches!(status, KeptVMStatus::Executed) {
                            let write_entries = rec.write_set.clone();
                            let ws_for_result = WriteSetMut::new(write_entries.clone())
                                .freeze()
                                .map_err(BlockExecutorError::BlockChainStateErr)?;

                            pending_reuse_writes.extend(write_entries);

                            let next_idx = idx + 1;
                            let need_apply_before_next = next_idx < transactions.len()
                                && !reused_indices.contains(&next_idx);
                            if need_apply_before_next || should_commit {
                                apply_pending_reuse_writes(statedb, &mut pending_reuse_writes)?;
                            }

                            let txn_state_root = if should_commit {
                                Some(
                                    statedb
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
                            reused = true;
                        }
                    }
                }
            }
        }

        if reused {
            continue;
        }

        if !pending_reuse_writes.is_empty() {
            apply_pending_reuse_writes(statedb, &mut pending_reuse_writes)?;
        }

        reuse_recorder::start();
        let vm_outputs = do_execute_block_transactions(
            statedb,
            vec![txn.clone()],
            Some(remaining_gas),
            vm_metrics.clone(),
        )
        .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        let mut read_sets = reuse_recorder::finish().into_iter();
        let output = vm_outputs
            .into_iter()
            .next()
            .ok_or(BlockExecutorError::BlockTransactionZero)?;

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
                let read_entries = read_sets.next().map(to_read_entries);
                let write_set_clone = write_set.clone();
                let write_entries_for_store =
                    write_set_clone.clone().into_iter().collect::<Vec<_>>();

                statedb
                    .apply_write_set(write_set)
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                let txn_state_root = if should_commit {
                    Some(
                        statedb
                            .commit()
                            .map_err(BlockExecutorError::BlockChainStateErr)?,
                    )
                } else {
                    None
                };

                let events_clone = events.clone();
                let txn_info = TransactionInfo::new(
                    tx_hash,
                    txn_state_root,
                    events.as_slice(),
                    gas_used,
                    keep_status.clone(),
                );
                let event_root = txn_info.event_root_hash();

                txn_infos.push(txn_info);
                txn_events.push(events_clone.clone());
                write_sets.push(write_set_clone.clone());

                remaining_gas = remaining_gas.saturating_sub(gas_used);

                let rec = ExecRecord {
                    tx_hash,
                    pre_state_fingerprint: pre_fp,
                    read_set: read_entries,
                    write_set: write_entries_for_store,
                    event_root,
                    gas: gas_used,
                    status_ok: matches!(keep_status, KeptVMStatus::Executed),
                    meta_fingerprint: None,
                    status: Some(keep_status),
                    events: events_clone,
                    table_infos: Vec::new(),
                };
                store.put(rec);
            }
        }
    }

    let mut executed = BlockExecutedData::default();
    executed.txn_infos = txn_infos;
    executed.txn_events = txn_events;
    executed.write_sets = write_sets;
    if !pending_reuse_writes.is_empty() {
        apply_pending_reuse_writes(statedb, &mut pending_reuse_writes)?;
    }
    executed.state_root = statedb.state_root();

    Ok(executed)
}

pub fn build_block_transactions(
    signed_txns: &[SignedUserTransaction],
    block_meta: Option<BlockMetadata>,
) -> Vec<Transaction> {
    let mut txns = block_meta
        .as_ref()
        .map(|m| vec![Transaction::BlockMetadata(m.clone())])
        .unwrap_or_default();
    txns.extend(
        signed_txns
            .iter()
            .map(|t| Transaction::UserTransaction(t.clone())),
    );

    // contains user transaction
    if txns.len() > 1 {
        let senders = signed_txns.iter().map(|t| t.sender()).collect();
        txns.extend(
            block_meta
                .map(|m| vec![Transaction::BlockEpilogue(m, senders)])
                .unwrap_or_default(),
        );
    }
    txns
}

/// Helper to compute pre-state fingerprint from parent root and BlockMetadata.
pub fn create_pre_state_fingerprint(
    parent_state_root2: starcoin_vm2_crypto::HashValue,
    metadata: &BlockMetadata2,
    epoch_version: u64,
) -> starcoin_vm2_crypto::HashValue {
    let meta_hash = metadata.crypto_hash();
    exec_merge::create_pre_state_fingerprint(parent_state_root2, meta_hash, epoch_version)
}

pub fn get_epoch_from_statedb(statedb: &ChainStateDB) -> anyhow::Result<Epoch> {
    let account_reader = AccountStateReader::new(statedb);
    account_reader.get_resource::<Epoch>(genesis_address())
}
