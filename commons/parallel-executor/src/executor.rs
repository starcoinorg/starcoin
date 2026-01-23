// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::Error::BlockRestart;
use crate::{
    delayed_field::{
        DelayedFieldRead, DelayedFieldReadKind, DelayedFieldReads, DelayedReadValidationError,
    },
    errors::*,
    scheduler::{Scheduler, SchedulerTask, TaskGuard, TxnIndex, Version},
    task::{ExecutionStatus, ExecutorTask, Transaction, TransactionOutput},
    txn_last_input_output::{ReadDescriptor, TxnLastInputOutput},
};
use move_vm_types::delayed_values::delayed_field_id::DelayedFieldID;
use num_cpus;
use once_cell::sync::Lazy;
use rand::Rng;
use starcoin_aggregator::types::{DelayedFieldsSpeculativeError, PanicOr};
use starcoin_infallible::Mutex;
use starcoin_logger::prelude::{error, info, log_enabled, Level};
use starcoin_mvhashmap::{
    versioned_delayed_fields::{CommitError, VersionedDelayedFields},
    MVHashMap,
};
use std::{
    collections::HashSet,
    hash::Hash,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
        Arc,
    },
    thread::spawn,
};
static RAYON_EXEC_POOL: Lazy<rayon::ThreadPool> = Lazy::new(|| {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .thread_name(|index| format!("parallel_executor_{}", index))
        .build()
        .unwrap()
});

fn gen_id_start_value(sequential: bool) -> u32 {
    let offset = if sequential { 0 } else { 1000 };
    let mut rng = rand::rng();
    let base: u32 = rng.random_range((1 + offset)..(1000 + offset));
    base.saturating_mul(1_000_000)
}

struct DelayedFieldCommitCoordinator {
    next_to_commit: AtomicUsize,
    fatal: AtomicBool,
}

impl DelayedFieldCommitCoordinator {
    fn new() -> Self {
        Self {
            next_to_commit: AtomicUsize::new(0),
            fatal: AtomicBool::new(false),
        }
    }

    fn advance(&self) {
        self.next_to_commit.fetch_add(1, Ordering::SeqCst);
    }

    fn next_to_commit(&self) -> usize {
        self.next_to_commit.load(Ordering::Acquire)
    }

    fn set_fatal(&self) {
        self.fatal.store(true, Ordering::Release);
    }

    fn is_fatal(&self) -> bool {
        self.fatal.load(Ordering::Acquire)
    }
}

fn set_fatal_error<E: Clone>(
    fatal_flag: &AtomicBool,
    fatal_error: &Mutex<Option<Error<E>>>,
    err: Error<E>,
    scheduler: &Scheduler,
    commit_coordinator: Option<&DelayedFieldCommitCoordinator>,
) {
    if !fatal_flag.swap(true, Ordering::SeqCst) {
        *fatal_error.lock() = Some(err);
    }
    if let Some(coord) = commit_coordinator {
        coord.set_fatal();
    }
    scheduler.force_done();
}

/// A struct that is always used by a single thread performing an execution task. The struct is
/// passed to the VM and acts as a proxy to resolve reads first in the shared multi-version
/// data-structure. It also allows the caller to track the read-set and any dependencies.
///
/// XXX FIXME YSG TODO(issue 10177): MvHashMapView currently needs to be sync due to trait bounds, but should
/// not be. In this case, the read_dependency member can have a RefCell<bool> type and the
/// captured_reads member can have RefCell<Vec<ReadDescriptor<K>>> type.
pub struct MVHashMapView<'a, K, V> {
    versioned_map: &'a MVHashMap<K, V>,
    txn_idx: TxnIndex,
    scheduler: &'a Scheduler,
    captured_reads: Mutex<Vec<ReadDescriptor<K>>>,
    captured_delayed_reads: Mutex<DelayedFieldReads>,
    delayed_field_id_start: u32,
    delayed_field_id_counter: &'a AtomicU32,
}

impl<K: PartialOrd + Send + Clone + Hash + Eq, V: Send + Sync> MVHashMapView<'_, K, V> {
    /// Drains the captured reads.
    pub fn take_reads(&self) -> Vec<ReadDescriptor<K>> {
        let mut reads = self.captured_reads.lock();
        std::mem::take(&mut reads)
    }

    pub fn take_delayed_field_reads(&self) -> DelayedFieldReads {
        let mut reads = self.captured_delayed_reads.lock();
        std::mem::take(&mut *reads)
    }

    pub fn capture_delayed_field_read(
        &self,
        id: DelayedFieldID,
        update: bool,
        read: DelayedFieldRead,
    ) -> std::result::Result<(), PanicOr<DelayedFieldsSpeculativeError>> {
        self.captured_delayed_reads
            .lock()
            .capture_delayed_field_read(id, update, read)
    }

    pub fn capture_delayed_field_read_error(&self, err: &PanicOr<DelayedFieldsSpeculativeError>) {
        self.captured_delayed_reads
            .lock()
            .capture_delayed_field_read_error(err);
    }

    pub fn get_delayed_field_by_kind(
        &self,
        id: &DelayedFieldID,
        min_kind: DelayedFieldReadKind,
    ) -> Option<DelayedFieldRead> {
        self.captured_delayed_reads
            .lock()
            .get_delayed_field_by_kind(id, min_kind)
    }

    pub fn generate_delayed_field_id(&self, width: u32) -> DelayedFieldID {
        let index = self.delayed_field_id_counter.fetch_add(1, Ordering::SeqCst);
        DelayedFieldID::new_with_width(index, width)
    }

    pub fn delayed_field_id_start(&self) -> u32 {
        self.delayed_field_id_start
    }

    pub fn delayed_field_id_counter(&self) -> u32 {
        self.delayed_field_id_counter.load(Ordering::SeqCst)
    }

    pub fn delayed_fields(&self) -> &VersionedDelayedFields<DelayedFieldID> {
        self.versioned_map.delayed_fields()
    }

    /// Captures a read from the VM execution.
    pub fn read(&self, key: &K) -> Option<Arc<V>> {
        loop {
            match self.versioned_map.read(key, self.txn_idx) {
                Ok((version, v)) => {
                    let (txn_idx, incarnation) = version;
                    self.captured_reads.lock().push(ReadDescriptor::from(
                        key.clone(),
                        txn_idx,
                        incarnation,
                    ));
                    return Some(v);
                }
                Err(None) => {
                    self.captured_reads
                        .lock()
                        .push(ReadDescriptor::from_storage(key.clone()));
                    return None;
                }
                Err(Some(dep_idx)) => {
                    // `self.txn_idx` estimated to depend on a write from `dep_idx`.
                    match self.scheduler.wait_for_dependency(self.txn_idx, dep_idx) {
                        Some(dep_condition) => {
                            // Wait on a condition variable corresponding to the encountered
                            // read dependency. Once the dep_idx finishes re-execution, scheduler
                            // will mark the dependency as resolved, and then the txn_idx will be
                            // scheduled for re-execution, which will re-awaken cvar here.
                            // A deadlock is not possible due to these condition variables:
                            // suppose all threads are waiting on read dependency, and consider
                            // one with lowest txn_idx. It observed a dependency, so some thread
                            // aborted dep_idx. If that abort returned execution task, by
                            // minimality (lower transactions aren't waiting), that thread would
                            // finish execution unblock txn_idx, contradiction. Otherwise,
                            // execution_idx in scheduler was lower at a time when at least the
                            // thread that aborted dep_idx was alive, and again, since lower txns
                            // than txn_idx are not blocked, so the execution of dep_idx will
                            // eventually finish and lead to unblocking txn_idx, contradiction.
                            let (lock, cvar) = &*dep_condition;
                            let mut dep_resolved = lock.lock();
                            while !*dep_resolved {
                                dep_resolved = cvar.wait(dep_resolved).unwrap();
                            }
                        }
                        None => continue,
                    }
                }
            };
        }
    }

    /// Return txn_idx associated with the MVHashMapView
    pub fn txn_idx(&self) -> TxnIndex {
        self.txn_idx
    }

    pub fn wait_for_dependency(&self, dep_idx: TxnIndex) -> bool {
        match self.scheduler.wait_for_dependency(self.txn_idx, dep_idx) {
            Some(dep_condition) => {
                let (lock, cvar) = &*dep_condition;
                let mut dep_resolved = lock.lock();
                while !*dep_resolved {
                    dep_resolved = cvar.wait(dep_resolved).unwrap();
                }
                true
            }
            None => false,
        }
    }
}
pub struct ParallelTransactionExecutor<T: Transaction, E: ExecutorTask> {
    // number of active concurrent tasks, corresponding to the maximum number of rayon
    // threads that may be concurrently participating in parallel execution.
    concurrency_level: usize,
    phantom: PhantomData<(T, E)>,
    gas_limit: Option<u64>,
    blockstm_v2: bool,
    delayed_fields_enabled: bool,
    delayed_field_id_start: u32,
    delayed_field_id_counter: AtomicU32,
}

struct ExecStats {
    exec_tasks: AtomicUsize,
    validate_tasks: AtomicUsize,
    aborts: AtomicUsize,
    execs_per_txn: Vec<AtomicUsize>,
}

impl ExecStats {
    fn new(num_txns: usize) -> Self {
        Self {
            exec_tasks: AtomicUsize::new(0),
            validate_tasks: AtomicUsize::new(0),
            aborts: AtomicUsize::new(0),
            execs_per_txn: (0..num_txns).map(|_| AtomicUsize::new(0)).collect(),
        }
    }

    fn record_exec(&self, idx: TxnIndex) {
        self.exec_tasks.fetch_add(1, Ordering::Relaxed);
        if let Some(slot) = self.execs_per_txn.get(idx) {
            slot.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn record_validate(&self) {
        self.validate_tasks.fetch_add(1, Ordering::Relaxed);
    }

    fn record_abort(&self) {
        self.aborts.fetch_add(1, Ordering::Relaxed);
    }
}

impl<T, E> ParallelTransactionExecutor<T, E>
where
    T: Transaction,
    E: ExecutorTask<T = T>,
{
    /// The caller needs to ensure that concurrency_level > 1 (0 is illegal and 1 should
    /// be handled by sequential execution) and that concurrency_level <= num_cpus.
    pub fn new(concurrency_level: usize, gas_limit: Option<u64>) -> Self {
        assert!(
            concurrency_level > 1 && concurrency_level <= num_cpus::get(),
            "Parallel execution concurrency level {} should be between 2 and number of CPUs",
            concurrency_level
        );
        let delayed_field_id_start = gen_id_start_value(false);
        Self {
            concurrency_level,
            phantom: PhantomData,
            gas_limit,
            blockstm_v2: false,
            delayed_fields_enabled: false,
            delayed_field_id_start,
            delayed_field_id_counter: AtomicU32::new(delayed_field_id_start),
        }
    }

    pub fn with_blockstm_v2(mut self, enabled: bool) -> Self {
        self.blockstm_v2 = enabled;
        self
    }

    pub fn with_delayed_fields(mut self, enabled: bool) -> Self {
        self.delayed_fields_enabled = enabled;
        self
    }

    fn execute<'a>(
        &self,
        version: Version,
        guard: TaskGuard<'a>,
        signature_verified_block: &[T],
        last_input_output: &TxnLastInputOutput<
            <T as Transaction>::Key,
            <E as ExecutorTask>::Output,
            <E as ExecutorTask>::Error,
        >,
        versioned_data_cache: &MVHashMap<<T as Transaction>::Key, <T as Transaction>::Value>,
        scheduler: &'a Scheduler,
        executor: &E,
        delayed_commit: Option<&DelayedFieldCommitCoordinator>,
        fatal_flag: &AtomicBool,
        fatal_error: &Mutex<Option<Error<E::Error>>>,
        stats: Option<&ExecStats>,
    ) -> SchedulerTask<'a> {
        let (idx_to_execute, incarnation) = version;
        let txn = &signature_verified_block[idx_to_execute];
        if txn.is_block_epilogue() {
            return scheduler.finish_execution(idx_to_execute, incarnation, false, guard);
        }
        if let Some(stats) = stats {
            stats.record_exec(idx_to_execute);
        }

        let state_view = MVHashMapView {
            versioned_map: versioned_data_cache,
            txn_idx: idx_to_execute,
            scheduler,
            captured_reads: Mutex::new(Vec::new()),
            captured_delayed_reads: Mutex::new(DelayedFieldReads::default()),
            delayed_field_id_start: self.delayed_field_id_start,
            delayed_field_id_counter: &self.delayed_field_id_counter,
        };

        // VM execution.
        let execute_result = executor.execute_transaction(&state_view, txn);
        let mut prev_write_set: HashSet<T::Key> = last_input_output.write_set(idx_to_execute);
        let mut prev_delayed_write_set = versioned_data_cache
            .delayed_fields()
            .get_txn_ids(idx_to_execute);

        // For tracking whether the recent execution wrote outside of the previous write set.
        let mut writes_outside = false;
        let mut apply_writes =
            |output: &<E as ExecutorTask>::Output| -> std::result::Result<(), ()> {
                let write_version = (idx_to_execute, incarnation);
                for (k, v) in output.get_writes().into_iter() {
                    if !prev_write_set.remove(&k) {
                        writes_outside = true
                    }
                    versioned_data_cache.write(&k, write_version, v);
                }
                for (id, change) in output.delayed_field_change_set().into_iter() {
                    if !prev_delayed_write_set.remove(&id) {
                        writes_outside = true;
                    }
                    let entry = change.into_entry_no_additional_history();
                    if let Err(err) = versioned_data_cache.delayed_fields().record_change(
                        id,
                        idx_to_execute,
                        entry,
                    ) {
                        match err {
                            PanicOr::CodeInvariantError(err_msg) => {
                                error!(
                                    "record_change failed: txn_idx={} id={:?} err={:?}",
                                    idx_to_execute, id, err_msg
                                );
                                set_fatal_error(
                                    fatal_flag,
                                    fatal_error,
                                    Error::InvariantViolation,
                                    scheduler,
                                    delayed_commit,
                                );
                                return Err(());
                            }
                            PanicOr::Or(_) => {
                                state_view.capture_delayed_field_read_error(&PanicOr::Or(
                                    DelayedFieldsSpeculativeError::InconsistentRead,
                                ));
                            }
                        }
                    }
                }
                Ok(())
            };

        let mut result = match execute_result {
            // These statuses are the results of speculative execution, so even for
            // SkipRest (skip the rest of transactions) and Abort (abort execution with
            // user defined error), no immediate action is taken. Instead the statuses
            // are recorded and (final statuses) are analyzed when the block is executed.
            ExecutionStatus::Success(output) => {
                // Apply the writes to the versioned_data_cache.
                if apply_writes(&output).is_err() {
                    ExecutionStatus::Abort(Error::InvariantViolation)
                } else {
                    ExecutionStatus::Success(output)
                }
            }
            ExecutionStatus::SkipRest(output) => {
                // Apply the writes and record status indicating skip.
                if apply_writes(&output).is_err() {
                    ExecutionStatus::Abort(Error::InvariantViolation)
                } else {
                    ExecutionStatus::SkipRest(output)
                }
            }
            ExecutionStatus::Abort(err) => {
                // Record the status indicating abort.
                ExecutionStatus::Abort(Error::UserError(err))
            }
        };

        // Remove entries from previous write set that were not overwritten.
        for k in &prev_write_set {
            versioned_data_cache.delete(k, idx_to_execute);
        }
        for id in &prev_delayed_write_set {
            if let Err(err) =
                versioned_data_cache
                    .delayed_fields()
                    .remove(id, idx_to_execute, self.blockstm_v2)
            {
                error!(
                    "remove delayed field failed: txn_idx={} id={:?} err={:?}",
                    idx_to_execute, id, err
                );
                set_fatal_error(
                    fatal_flag,
                    fatal_error,
                    Error::InvariantViolation,
                    scheduler,
                    delayed_commit,
                );
                result = ExecutionStatus::Abort(Error::InvariantViolation);
                break;
            }
        }

        let reads = state_view.take_reads();
        let delayed_reads = state_view.take_delayed_field_reads();
        last_input_output.record(idx_to_execute, reads, delayed_reads, result);
        scheduler.finish_execution(idx_to_execute, incarnation, writes_outside, guard)
    }

    fn validate<'a>(
        &self,
        version_to_validate: Version,
        guard: TaskGuard<'a>,
        signature_verified_block: &[T],
        last_input_output: &TxnLastInputOutput<
            <T as Transaction>::Key,
            <E as ExecutorTask>::Output,
            <E as ExecutorTask>::Error,
        >,
        versioned_data_cache: &MVHashMap<<T as Transaction>::Key, <T as Transaction>::Value>,
        scheduler: &'a Scheduler,
        _delayed_commit: Option<&DelayedFieldCommitCoordinator>,
        fatal_flag: &AtomicBool,
        fatal_error: &Mutex<Option<Error<E::Error>>>,
        stats: Option<&ExecStats>,
    ) -> SchedulerTask<'a> {
        if fatal_flag.load(Ordering::Acquire) {
            scheduler.force_done();
            return SchedulerTask::Done;
        }
        let (idx_to_validate, incarnation) = version_to_validate;
        if let Some(stats) = stats {
            stats.record_validate();
        }
        if signature_verified_block[idx_to_validate].is_block_epilogue() {
            return SchedulerTask::NoTask;
        }
        if !scheduler.is_executed_incarnation(idx_to_validate, incarnation) {
            return SchedulerTask::NoTask;
        }
        let read_set = last_input_output
            .read_set(idx_to_validate)
            .expect("Prior read-set must be recorded");

        if let Some(delayed_reads) = last_input_output.delayed_read_set(idx_to_validate) {
            if delayed_reads.is_incorrect_use() {
                set_fatal_error(
                    fatal_flag,
                    fatal_error,
                    Error::InvariantViolation,
                    scheduler,
                    _delayed_commit,
                );
                return SchedulerTask::Done;
            }
            if delayed_reads.has_speculative_failure() {
                // Speculative delayed-field read failure requires re-execution.
                let aborted = scheduler.try_abort(idx_to_validate, incarnation);
                if aborted {
                    for k in &last_input_output.write_set(idx_to_validate) {
                        versioned_data_cache.mark_estimate(k, idx_to_validate);
                    }
                    let delayed_ids = versioned_data_cache
                        .delayed_fields()
                        .get_txn_ids(idx_to_validate);
                    for id in &delayed_ids {
                        versioned_data_cache
                            .delayed_fields()
                            .mark_estimate(id, idx_to_validate);
                    }
                    return scheduler.finish_abort(idx_to_validate, incarnation, guard);
                }
            }
        }

        let read_valid = read_set.iter().all(|r| {
            match versioned_data_cache.read(r.path(), idx_to_validate) {
                Ok((version, _)) => r.validate_version(version),
                Err(Some(_)) => false, // Dependency implies a validation failure.
                Err(None) => r.validate_storage(),
            }
        });
        let valid = read_valid;
        if !valid {
            info!(
                target: "starcoin_parallel_executor",
                "validate failed txn_idx={} incarnation={} read_valid={}",
                idx_to_validate,
                incarnation,
                read_valid
            );
        }

        let aborted = !valid && scheduler.try_abort(idx_to_validate, incarnation);

        if aborted {
            info!(
                target: "starcoin_parallel_executor",
                "abort txn_idx={} incarnation={}",
                idx_to_validate,
                incarnation
            );
            if let Some(stats) = stats {
                stats.record_abort();
            }
            // Not valid and successfully aborted, mark the latest write-set as estimates.
            for k in &last_input_output.write_set(idx_to_validate) {
                versioned_data_cache.mark_estimate(k, idx_to_validate);
            }
            let delayed_ids = versioned_data_cache
                .delayed_fields()
                .get_txn_ids(idx_to_validate);
            for id in &delayed_ids {
                versioned_data_cache
                    .delayed_fields()
                    .mark_estimate(id, idx_to_validate);
            }
            scheduler.finish_abort(idx_to_validate, incarnation, guard)
        } else {
            // Get gas_used from the transaction output
            let gas_used = last_input_output.gas_used(idx_to_validate);
            scheduler.finish_validation(idx_to_validate, gas_used);
            SchedulerTask::NoTask
        }
    }

    fn work_task_with_scope(
        &self,
        executor_arguments: &E::Argument,
        block: &[T],
        last_input_output: &TxnLastInputOutput<
            <T as Transaction>::Key,
            <E as ExecutorTask>::Output,
            <E as ExecutorTask>::Error,
        >,
        versioned_data_cache: &MVHashMap<<T as Transaction>::Key, <T as Transaction>::Value>,
        scheduler: &Scheduler,
        delayed_commit: Option<Arc<DelayedFieldCommitCoordinator>>,
        fatal_flag: Arc<AtomicBool>,
        fatal_error: Arc<Mutex<Option<Error<E::Error>>>>,
        stats: Option<Arc<ExecStats>>,
    ) {
        // Make executor for each task. TODO: fast concurrent executor.
        let executor = E::init(executor_arguments.clone());
        let stats = stats.as_deref();

        let mut scheduler_task = SchedulerTask::NoTask;
        loop {
            if fatal_flag.load(Ordering::Acquire) {
                scheduler.force_done();
                break;
            }
            scheduler_task = match scheduler_task {
                SchedulerTask::ValidationTask(version_to_validate, guard) => self.validate(
                    version_to_validate,
                    guard,
                    block,
                    last_input_output,
                    versioned_data_cache,
                    scheduler,
                    delayed_commit.as_deref(),
                    fatal_flag.as_ref(),
                    fatal_error.as_ref(),
                    stats,
                ),
                SchedulerTask::ExecutionTask(version_to_execute, None, guard) => self.execute(
                    version_to_execute,
                    guard,
                    block,
                    last_input_output,
                    versioned_data_cache,
                    scheduler,
                    &executor,
                    delayed_commit.as_deref(),
                    fatal_flag.as_ref(),
                    fatal_error.as_ref(),
                    stats,
                ),
                SchedulerTask::ExecutionTask(_, Some(condvar), _guard) => {
                    let (lock, cvar) = &*condvar;
                    // Mark dependency resolved.
                    *lock.lock() = true;
                    // Wake up the process waiting for dependency.
                    cvar.notify_one();

                    SchedulerTask::NoTask
                }
                SchedulerTask::NoTask => scheduler.next_task(),
                SchedulerTask::Done => {
                    break;
                }
            }
        }
    }

    fn execute_block_prologue(
        &self,
        executor_arguments: &E::Argument,
        block: &[T],
        last_input_output: &TxnLastInputOutput<
            <T as Transaction>::Key,
            <E as ExecutorTask>::Output,
            <E as ExecutorTask>::Error,
        >,
        versioned_data_cache: &MVHashMap<<T as Transaction>::Key, <T as Transaction>::Value>,
        scheduler: &Scheduler,
        delayed_commit: Option<&DelayedFieldCommitCoordinator>,
        fatal_flag: &AtomicBool,
        fatal_error: &Mutex<Option<Error<E::Error>>>,
        stats: Option<&ExecStats>,
    ) {
        if block.is_empty() || !block[0].is_block_prologue() {
            return;
        }

        let executor = E::init(executor_arguments.clone());
        match scheduler.next_task() {
            SchedulerTask::ExecutionTask(version, None, guard) => {
                let (idx_to_execute, incarnation) = version;
                assert!(idx_to_execute == 0 && incarnation == 0);
                self.execute(
                    version,
                    guard,
                    block,
                    last_input_output,
                    versioned_data_cache,
                    scheduler,
                    &executor,
                    delayed_commit,
                    fatal_flag,
                    fatal_error,
                    stats,
                );
            }
            _ => {
                unreachable!()
            }
        };

        match scheduler.next_task() {
            SchedulerTask::ValidationTask(version, guard) => {
                let (idx_to_execute, incarnation) = version;
                assert!(idx_to_execute == 0 && incarnation == 0);
                self.validate(
                    version,
                    guard,
                    block,
                    last_input_output,
                    versioned_data_cache,
                    scheduler,
                    delayed_commit,
                    fatal_flag,
                    fatal_error,
                    stats,
                );
            }
            _ => {
                error!("second task from scheduler should be validation of block metadata txn, maybe execute block meta txn failed?");
            }
        };
    }

    fn execute_block_epilogue(
        &self,
        executor_arguments: &E::Argument,
        block: &[T],
        last_input_output: &TxnLastInputOutput<
            <T as Transaction>::Key,
            <E as ExecutorTask>::Output,
            <E as ExecutorTask>::Error,
        >,
        versioned_data_cache: &MVHashMap<<T as Transaction>::Key, <T as Transaction>::Value>,
        scheduler: &Scheduler,
        stats: Option<&ExecStats>,
    ) {
        // Check if block has epilogue
        if block.is_empty() || !block[block.len() - 1].is_block_epilogue() {
            return;
        }

        // Get the first index that exceeds gas limit
        let epilogue_idx = block.len() - 1;
        if let Some(stats) = stats {
            stats.record_exec(epilogue_idx);
        }
        let view_index = scheduler.first_exceeding_index().min(epilogue_idx);
        let state_view = MVHashMapView {
            versioned_map: versioned_data_cache,
            txn_idx: view_index, // This makes it see state after view_index-1
            scheduler,
            captured_reads: Mutex::new(Vec::new()),
            captured_delayed_reads: Mutex::new(DelayedFieldReads::default()),
            delayed_field_id_start: self.delayed_field_id_start,
            delayed_field_id_counter: &self.delayed_field_id_counter,
        };

        // Execute block epilogue with the snapshot view
        let executor = E::init(executor_arguments.clone());
        let execute_result = executor.execute_transaction(&state_view, &block[epilogue_idx]);

        // Apply the writes from epilogue execution
        let apply_writes = |output: &<E as ExecutorTask>::Output| -> std::result::Result<(), ()> {
            let write_version = (epilogue_idx, 0);
            for (k, v) in output.get_writes().into_iter() {
                versioned_data_cache.write(&k, write_version, v);
            }
            for (id, change) in output.delayed_field_change_set().into_iter() {
                let entry = change.into_entry_no_additional_history();
                if let Err(err) =
                    versioned_data_cache
                        .delayed_fields()
                        .record_change(id, epilogue_idx, entry)
                {
                    error!(
                        "record_change failed (epilogue): txn_idx={} id={:?} err={:?}",
                        epilogue_idx, id, err
                    );
                    return Err(());
                }
            }
            Ok(())
        };

        let result = match execute_result {
            ExecutionStatus::Success(output) => {
                if apply_writes(&output).is_err() {
                    ExecutionStatus::Abort(Error::InvariantViolation)
                } else {
                    ExecutionStatus::Success(output)
                }
            }
            ExecutionStatus::SkipRest(_output) => ExecutionStatus::Abort(Error::InvariantViolation),
            ExecutionStatus::Abort(err) => ExecutionStatus::Abort(Error::UserError(err)),
        };

        // Record the epilogue execution result
        let reads = state_view.take_reads();
        let delayed_reads = state_view.take_delayed_field_reads();
        last_input_output.record(epilogue_idx, reads, delayed_reads, result);
    }

    pub fn execute_transactions_parallel(
        &self,
        executor_initial_arguments: E::Argument,
        signature_verified_block: Vec<T>,
    ) -> Result<Vec<E::Output>, E::Error> {
        let (outputs, _delayed_fields) = self.execute_transactions_parallel_with_delayed_fields(
            executor_initial_arguments,
            signature_verified_block,
        )?;
        let mut outputs = outputs;
        outputs.sort_by_key(|(idx, _)| *idx);
        Ok(outputs.into_iter().map(|(_, output)| output).collect())
    }

    pub fn execute_transactions_parallel_with_delayed_fields(
        &self,
        executor_initial_arguments: E::Argument,
        signature_verified_block: Vec<T>,
    ) -> Result<
        (
            Vec<(TxnIndex, E::Output)>,
            VersionedDelayedFields<DelayedFieldID>,
        ),
        E::Error,
    > {
        if signature_verified_block.is_empty() {
            return Ok((vec![], VersionedDelayedFields::empty()));
        }

        let num_txns = signature_verified_block.len();
        let stats = if log_enabled!(target: "vm2-blockbench", Level::Info) {
            Some(Arc::new(ExecStats::new(num_txns)))
        } else {
            None
        };
        let versioned_data_cache = MVHashMap::new();
        let last_input_output = TxnLastInputOutput::new(num_txns);
        let scheduler = Scheduler::new(num_txns, self.gas_limit);
        let delayed_commit = if self.delayed_fields_enabled {
            Some(Arc::new(DelayedFieldCommitCoordinator::new()))
        } else {
            None
        };
        let fatal_flag = Arc::new(AtomicBool::new(false));
        let fatal_error: Arc<Mutex<Option<Error<E::Error>>>> = Arc::new(Mutex::new(None));

        // BlockMetadata is always the first txn of block that modifies fundamental info of block
        // other txns depends on this execution result, execute it first to avoid unnecessary contention
        self.execute_block_prologue(
            &executor_initial_arguments,
            &signature_verified_block,
            &last_input_output,
            &versioned_data_cache,
            &scheduler,
            delayed_commit.as_deref(),
            fatal_flag.as_ref(),
            fatal_error.as_ref(),
            stats.as_deref(),
        );

        RAYON_EXEC_POOL.scope(|s| {
            for _ in 0..self.concurrency_level {
                let delayed_commit = delayed_commit.clone();
                let fatal_flag = fatal_flag.clone();
                let fatal_error = fatal_error.clone();
                let stats = stats.clone();
                s.spawn(|_| {
                    self.work_task_with_scope(
                        &executor_initial_arguments,
                        &signature_verified_block,
                        &last_input_output,
                        &versioned_data_cache,
                        &scheduler,
                        delayed_commit,
                        fatal_flag,
                        fatal_error,
                        stats,
                    );
                });
            }
        });

        // scheduler may skip txns, block epilogue must be explictly executed
        self.execute_block_epilogue(
            &executor_initial_arguments,
            &signature_verified_block,
            &last_input_output,
            &versioned_data_cache,
            &scheduler,
            stats.as_deref(),
        );

        // Get the actual number of transactions to execute (considering gas limit)
        let first_exceeding = scheduler.first_exceeding_index();
        let num_txns_to_collect = first_exceeding.min(scheduler.num_txn_to_execute());

        // Check if there's a block epilogue
        let has_epilogue = !signature_verified_block.is_empty()
            && signature_verified_block[signature_verified_block.len() - 1].is_block_epilogue();

        let mut maybe_err = None;
        if fatal_flag.load(Ordering::Acquire) {
            maybe_err = fatal_error.lock().take();
        }

        if maybe_err.is_none() && self.delayed_fields_enabled {
            if let Some(coord) = delayed_commit.as_deref() {
                if coord.is_fatal() {
                    maybe_err = fatal_error
                        .lock()
                        .take()
                        .or(Some(Error::InvariantViolation));
                } else {
                    let delayed_fields = versioned_data_cache.delayed_fields();
                    let start_idx = coord.next_to_commit();
                    let epilogue_idx = if has_epilogue {
                        signature_verified_block.len() - 1
                    } else {
                        usize::MAX
                    };
                    for idx in start_idx..signature_verified_block.len() {
                        if coord.is_fatal() {
                            maybe_err = fatal_error
                                .lock()
                                .take()
                                .or(Some(Error::InvariantViolation));
                            break;
                        }
                        let should_commit =
                            idx < num_txns_to_collect || (has_epilogue && idx == epilogue_idx);
                        if should_commit {
                            if let Some(reads) = last_input_output.delayed_read_set(idx) {
                                if reads.is_incorrect_use() {
                                    coord.set_fatal();
                                    maybe_err = Some(Error::InvariantViolation);
                                    break;
                                }
                                match reads.validate_delayed_field_reads(delayed_fields, idx) {
                                    Ok(()) => {}
                                    Err(DelayedReadValidationError::Invalid) => {
                                        maybe_err = Some(BlockRestart);
                                        break;
                                    }
                                    Err(DelayedReadValidationError::Fatal) => {
                                        coord.set_fatal();
                                        maybe_err = Some(Error::InvariantViolation);
                                        break;
                                    }
                                }
                            }
                            let delayed_write_set = delayed_fields.take_txn_ids(idx);
                            match delayed_fields.try_commit(idx, delayed_write_set.into_iter()) {
                                Ok(()) => coord.advance(),
                                Err(CommitError::ReExecutionNeeded(_)) => {
                                    maybe_err = Some(BlockRestart);
                                    break;
                                }
                                Err(CommitError::CodeInvariantError(_)) => {
                                    coord.set_fatal();
                                    maybe_err = Some(Error::InvariantViolation);
                                    break;
                                }
                            }
                        } else {
                            let delayed_ids = delayed_fields.take_txn_ids(idx);
                            for id in &delayed_ids {
                                delayed_fields.mark_estimate(id, idx);
                                if let Err(err) = delayed_fields.remove(id, idx, self.blockstm_v2) {
                                    error!(
                                        "discard delayed field failed: txn_idx={} id={:?} err={:?}",
                                        idx, id, err
                                    );
                                    coord.set_fatal();
                                    maybe_err = Some(Error::InvariantViolation);
                                    break;
                                }
                            }
                            if maybe_err.is_some() {
                                break;
                            }
                            match delayed_fields.try_commit(idx, std::iter::empty()) {
                                Ok(()) => coord.advance(),
                                Err(CommitError::ReExecutionNeeded(_)) => {
                                    maybe_err = Some(BlockRestart);
                                    break;
                                }
                                Err(CommitError::CodeInvariantError(_)) => {
                                    coord.set_fatal();
                                    maybe_err = Some(Error::InvariantViolation);
                                    break;
                                }
                            }
                        }
                    }
                }
            } else {
                maybe_err = Some(Error::InvariantViolation);
            }
        }

        if maybe_err.is_none() && fatal_flag.load(Ordering::Acquire) {
            maybe_err = fatal_error
                .lock()
                .take()
                .or(Some(Error::InvariantViolation));
        }

        // TODO: for large block sizes and many cores, extract outputs in parallel.
        let mut final_results = Vec::with_capacity(num_txns_to_collect + 1);

        if maybe_err.is_none() {
            // Collect all outputs from transactions within gas limit (0..num_txns_to_collect)
            for idx in 0..num_txns_to_collect {
                match last_input_output.take_output(idx) {
                    ExecutionStatus::Success(t) => final_results.push((idx, t)),
                    ExecutionStatus::SkipRest(_t) => {
                        maybe_err = Some(BlockRestart);
                        break;
                    }
                    ExecutionStatus::Abort(err) => {
                        maybe_err = Some(err);
                        break;
                    }
                };
            }

            // Collect epilogue output if it exists and no error occurred
            // Epilogue is at the end of the block, so its index might be > num_txns_to_collect
            if has_epilogue && maybe_err.is_none() {
                let epilogue_idx = signature_verified_block.len() - 1;
                // Only collect epilogue if it's not already collected above
                if epilogue_idx >= num_txns_to_collect {
                    match last_input_output.take_output(epilogue_idx) {
                        ExecutionStatus::Success(t) => final_results.push((epilogue_idx, t)),
                        ExecutionStatus::SkipRest(_t) => {
                            maybe_err = Some(BlockRestart);
                        }
                        ExecutionStatus::Abort(err) => {
                            maybe_err = Some(err);
                        }
                    };
                }
            }
        }

        let delayed_fields = versioned_data_cache.into_delayed_fields();
        spawn(move || {
            // Explicit async drops.
            drop(last_input_output);
            drop(signature_verified_block);
            drop(scheduler);
        });

        if let Some(stats) = stats.as_ref() {
            let exec_tasks = stats.exec_tasks.load(Ordering::Relaxed);
            let validate_tasks = stats.validate_tasks.load(Ordering::Relaxed);
            let aborts = stats.aborts.load(Ordering::Relaxed);
            let mut unique_exec_txns = 0usize;
            let mut reexec_txns = 0usize;
            let mut reexec_total = 0usize;
            let mut max_execs = 0usize;
            for counter in &stats.execs_per_txn {
                let count = counter.load(Ordering::Relaxed);
                if count > 0 {
                    unique_exec_txns += 1;
                    if count > 1 {
                        reexec_txns += 1;
                        reexec_total += count - 1;
                    }
                    if count > max_execs {
                        max_execs = count;
                    }
                }
            }
            info!(
                target: "vm2-blockbench",
                "parallel_exec.stats exec_tasks={} validate_tasks={} aborts={} unique_exec_txns={} reexec_txns={} reexec_total={} max_execs={}",
                exec_tasks,
                validate_tasks,
                aborts,
                unique_exec_txns,
                reexec_txns,
                reexec_total,
                max_execs
            );
        }

        match maybe_err {
            Some(err) => Err(err),
            None => Ok((final_results, delayed_fields)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use move_vm_types::delayed_values::delayed_field_id::DelayedFieldID;
    use starcoin_aggregator::delayed_change::DelayedChange;
    use starcoin_aggregator::types::{
        DelayedFieldValue, DelayedFieldsSpeculativeError, PanicOr, ReadPosition,
    };
    use starcoin_mvhashmap::{
        types::MVDelayedFieldsError, versioned_delayed_fields::TVersionedDelayedFieldView,
    };
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Condvar};

    // Minimal transaction implementation for testing
    #[derive(Debug, Clone)]
    struct TestTransaction {
        key: String,
        value: u64,
        acquire_cvar: Option<Arc<(Mutex<bool>, Condvar)>>,
        release_cvar: Option<Arc<(Mutex<bool>, Condvar)>>,
    }

    impl TestTransaction {
        fn simple(key: &str, value: u64) -> Self {
            Self {
                key: key.to_string(),
                value,
                acquire_cvar: None,
                release_cvar: None,
            }
        }

        fn wait(&self) {
            if let Some(arc) = &self.acquire_cvar {
                let (lock, cvar) = &**arc;
                let mut resolved = lock.lock();
                while !*resolved {
                    resolved = cvar.wait(resolved).unwrap();
                }
            }
        }

        fn notify(&self) {
            if let Some(arc) = self.release_cvar.as_ref() {
                let (lock, cvar) = &**arc;
                let mut resolved = lock.lock();
                *resolved = true;
                cvar.notify_all();
            }
        }
    }

    impl Transaction for TestTransaction {
        type Key = String;
        type Value = u64;
    }

    // Generic test output that can track both writes and gas
    #[derive(Debug, Clone)]
    struct TestOutput {
        writes: HashMap<String, u64>,
        gas: u64,
    }

    impl TransactionOutput for TestOutput {
        type T = TestTransaction;

        fn get_writes(&self) -> Vec<(String, u64)> {
            self.writes.iter().map(|(k, v)| (k.clone(), *v)).collect()
        }

        fn gas_used(&self) -> u64 {
            self.gas
        }

        fn skip_output() -> Self {
            Self {
                writes: HashMap::new(),
                gas: 0,
            }
        }
    }

    // Generic test executor with configurable behavior
    struct TestExecutor {
        initial_value: u64,
        gas_mode: GasMode,
    }

    #[derive(Clone, Copy)]
    enum GasMode {
        Fixed(u64), // Fixed gas per transaction
        UseValue,   // Use transaction value as gas
    }

    impl ExecutorTask for TestExecutor {
        type T = TestTransaction;
        type Output = TestOutput;
        type Error = ();
        type Argument = (u64, GasMode);

        fn init(args: Self::Argument) -> Self {
            Self {
                initial_value: args.0,
                gas_mode: args.1,
            }
        }

        fn execute_transaction(
            &self,
            view: &MVHashMapView<String, u64>,
            txn: &Self::T,
        ) -> ExecutionStatus<Self::Output, ()> {
            txn.wait();

            let current_value = view
                .read(&txn.key)
                .map(|v| *v)
                .unwrap_or(self.initial_value);

            let new_value = current_value + txn.value;
            let mut writes = HashMap::new();
            writes.insert(txn.key.clone(), new_value);

            let gas = match self.gas_mode {
                GasMode::Fixed(g) => g,
                GasMode::UseValue => txn.value,
            };

            txn.notify();

            ExecutionStatus::Success(TestOutput { writes, gas })
        }
    }

    #[derive(Debug, Clone)]
    struct SeqTransaction {
        idx: usize,
    }

    impl Transaction for SeqTransaction {
        type Key = usize;
        type Value = usize;
    }

    #[derive(Debug, Clone)]
    struct SeqOutput;

    impl TransactionOutput for SeqOutput {
        type T = SeqTransaction;

        fn get_writes(&self) -> Vec<(usize, usize)> {
            vec![]
        }

        fn gas_used(&self) -> u64 {
            0
        }

        fn skip_output() -> Self {
            Self
        }
    }

    struct SeqExecutor;

    impl ExecutorTask for SeqExecutor {
        type T = SeqTransaction;
        type Output = SeqOutput;
        type Error = ();
        type Argument = ();

        fn init(_args: Self::Argument) -> Self {
            Self
        }

        fn execute_transaction(
            &self,
            _view: &MVHashMapView<usize, usize>,
            txn: &Self::T,
        ) -> ExecutionStatus<Self::Output, ()> {
            info!("parallel execute txn idx={}", txn.idx);
            ExecutionStatus::Success(SeqOutput)
        }
    }

    #[test]
    fn test_parallel_conflicting_transactions() {
        // Create two transactions that will conflict (both modify the same key)
        let cvar = Arc::new((Mutex::new(false), Condvar::new()));
        let transactions = vec![
            TestTransaction {
                key: "shared_counter".to_string(),
                value: 10,
                acquire_cvar: Some(cvar.clone()),
                release_cvar: None,
            },
            TestTransaction {
                key: "shared_counter".to_string(),
                value: 20,
                acquire_cvar: None,
                release_cvar: Some(cvar.clone()),
            },
        ];

        let initial_value = 0;
        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(num_cpus::get().max(2), None);

        let result = executor
            .execute_transactions_parallel((initial_value, GasMode::Fixed(100)), transactions);
        assert!(
            result.is_ok(),
            "Conflicting transactions should still succeed"
        );

        let outputs: Vec<TestOutput> = result.unwrap();
        assert_eq!(outputs.len(), 2);

        // Verify that both transactions produced outputs
        for (i, output) in outputs.iter().enumerate() {
            assert!(
                output.writes.contains_key("shared_counter"),
                "Transaction {} should write to shared_counter",
                i
            );
        }

        // The final result should be the cumulative effect: 0 + 10 + 20 = 30
        let final_output = &outputs[1];
        if let Some(final_value) = final_output.writes.get("shared_counter") {
            assert_eq!(*final_value, 30, "Final value should be 30 (0 + 10 + 20)");
        }
    }

    #[test]
    fn test_parallel_execution_can_start_out_of_order() {
        starcoin_logger::init_for_test();
        let count = 50;
        let transactions: Vec<SeqTransaction> =
            (0..count).map(|idx| SeqTransaction { idx }).collect();
        let executor: ParallelTransactionExecutor<SeqTransaction, SeqExecutor> =
            ParallelTransactionExecutor::new(num_cpus::get().max(2), None);

        executor
            .execute_transactions_parallel((), transactions)
            .expect("parallel execute");
    }

    #[test]
    fn test_parallel_independent_transactions() {
        // Create two transactions that operate on different keys (no conflicts)
        let transactions = vec![
            TestTransaction::simple("account_a", 100),
            TestTransaction::simple("account_b", 200),
        ];

        let initial_value = 50;
        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(num_cpus::get().max(2), None);

        let result = executor
            .execute_transactions_parallel((initial_value, GasMode::Fixed(100)), transactions);
        assert!(result.is_ok(), "Independent transactions should succeed");

        let outputs: Vec<TestOutput> = result.unwrap();
        assert_eq!(outputs.len(), 2);

        // Verify that both transactions produced outputs with correct values
        let output_a = &outputs[0];
        let output_b = &outputs[1];

        // First transaction: account_a should have initial_value + 100 = 150
        assert!(
            output_a.writes.contains_key("account_a"),
            "First transaction should write to account_a"
        );
        if let Some(value_a) = output_a.writes.get("account_a") {
            assert_eq!(*value_a, 150, "account_a should have value 150 (50 + 100)");
        }

        // Second transaction: account_b should have initial_value + 200 = 250
        assert!(
            output_b.writes.contains_key("account_b"),
            "Second transaction should write to account_b"
        );
        if let Some(value_b) = output_b.writes.get("account_b") {
            assert_eq!(*value_b, 250, "account_b should have value 250 (50 + 200)");
        }

        // Verify no cross-contamination
        assert!(
            !output_a.writes.contains_key("account_b"),
            "First transaction should not write to account_b"
        );
        assert!(
            !output_b.writes.contains_key("account_a"),
            "Second transaction should not write to account_a"
        );
    }

    // ========== Gas Limit Tests ==========

    #[test]
    fn test_gas_limit_all_transactions_fit() {
        // Gas: 100, 200, 300, total = 600, limit = 1000
        let transactions = vec![
            TestTransaction::simple("key1", 100),
            TestTransaction::simple("key2", 200),
            TestTransaction::simple("key3", 300),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, Some(1000));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3, "All transactions should execute");
    }

    #[test]
    fn test_gas_limit_some_transactions_excluded() {
        // Gas: 400, 300, 500, total = 1200, limit = 800
        // Should execute: txn0 (400) + txn1 (300) = 700, stop at txn2
        let transactions = vec![
            TestTransaction::simple("key1", 400),
            TestTransaction::simple("key2", 300),
            TestTransaction::simple("key3", 500),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, Some(800));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().len(),
            2,
            "Only first 2 transactions should execute"
        );
    }

    #[test]
    fn test_gas_limit_first_transaction_exceeds() {
        // First transaction itself exceeds limit
        let transactions = vec![
            TestTransaction::simple("key1", 1500),
            TestTransaction::simple("key2", 100),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, Some(1000));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0, "No transactions should execute");
    }

    #[test]
    fn test_gas_limit_exact_boundary() {
        // Exact gas limit: 500 + 500 = 1000
        let transactions = vec![
            TestTransaction::simple("key1", 500),
            TestTransaction::simple("key2", 500),
            TestTransaction::simple("key3", 1),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, Some(1000));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().len(),
            2,
            "First 2 transactions should execute exactly"
        );
    }

    #[test]
    fn test_gas_limit_zero_gas_transactions() {
        // Mix of zero and non-zero gas
        let transactions = vec![
            TestTransaction::simple("key1", 0),
            TestTransaction::simple("key2", 500),
            TestTransaction::simple("key3", 0),
            TestTransaction::simple("key4", 600),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, Some(1000));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 3, "First 3 transactions should execute");
    }

    #[test]
    fn test_no_gas_limit() {
        // No gas limit means all transactions execute
        let transactions = vec![
            TestTransaction::simple("key1", 1000),
            TestTransaction::simple("key2", 2000),
            TestTransaction::simple("key3", 3000),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, None);

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().len(),
            3,
            "All transactions should execute without limit"
        );
    }

    // ========== Parallel Execution Edge Cases ==========

    #[test]
    fn test_empty_transaction_list() {
        let transactions: Vec<TestTransaction> = vec![];
        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, Some(1000));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_single_transaction() {
        let transactions = vec![TestTransaction::simple("key1", 100)];
        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(4, Some(1000));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_many_conflicting_transactions() {
        // 10 transactions all modifying the same key
        let transactions: Vec<TestTransaction> = (0..10)
            .map(|i| TestTransaction::simple("shared_key", 50 + i * 10))
            .collect();

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(4, None);

        let result = executor.execute_transactions_parallel((0, GasMode::Fixed(100)), transactions);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().len(),
            10,
            "All conflicting transactions should execute"
        );
    }

    #[test]
    fn test_many_independent_transactions() {
        // 20 transactions on different keys
        let transactions: Vec<TestTransaction> = (0..20)
            .map(|i| TestTransaction::simple(&format!("key{}", i), 100))
            .collect();

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(4, None);

        let result = executor.execute_transactions_parallel((0, GasMode::Fixed(100)), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 20);
    }

    #[test]
    fn test_mixed_conflict_pattern() {
        // Pattern: txn0,txn1 conflict on keyA; txn2,txn3 conflict on keyB; txn4 independent
        let transactions = vec![
            TestTransaction::simple("keyA", 100),
            TestTransaction::simple("keyA", 150),
            TestTransaction::simple("keyB", 200),
            TestTransaction::simple("keyB", 250),
            TestTransaction::simple("keyC", 300),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(4, None);

        let result = executor.execute_transactions_parallel((0, GasMode::Fixed(100)), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }

    #[test]
    fn test_gas_limit_with_conflicts() {
        // Conflicting transactions with gas limit
        // txn0: key1, gas=300; txn1: key1, gas=400; txn2: key2, gas=500
        // Limit=800, should execute txn0+txn1=700
        let transactions = vec![
            TestTransaction::simple("key1", 300),
            TestTransaction::simple("key1", 400),
            TestTransaction::simple("key2", 500),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(4, Some(800));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_high_concurrency() {
        // Test with high concurrency (capped at num_cpus)
        let transactions = vec![
            TestTransaction::simple("key1", 100),
            TestTransaction::simple("key2", 200),
        ];

        let concurrency = num_cpus::get().min(8); // Use available CPUs, max 8
        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(concurrency, None);

        let result = executor.execute_transactions_parallel((0, GasMode::Fixed(100)), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_low_concurrency() {
        // Test with minimum concurrency (2 threads)
        let transactions = vec![
            TestTransaction::simple("key1", 100),
            TestTransaction::simple("key2", 200),
            TestTransaction::simple("key3", 300),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, None);

        let result = executor.execute_transactions_parallel((0, GasMode::Fixed(100)), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_gas_limit_decreasing_gas_pattern() {
        // Decreasing gas: 500, 300, 100, 1, total = 901, limit=900
        // Should execute first 3: 500+300+100=900 (within limit)
        // txn3 would make it 901, so it's excluded
        let transactions = vec![
            TestTransaction::simple("key1", 500),
            TestTransaction::simple("key2", 300),
            TestTransaction::simple("key3", 100),
            TestTransaction::simple("key4", 1),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, Some(900));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        // All 4 transactions will execute because 500+300+100+1=901 only slightly exceeds
        // Actually, only first 3 are valid: 500+300+100=900 <= 900, adding 1 makes 901 > 900
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_gas_limit_increasing_gas_pattern() {
        // Increasing gas: 100, 200, 300, 400, limit=650
        // Should execute: 100+200+300=600
        let transactions = vec![
            TestTransaction::simple("key1", 100),
            TestTransaction::simple("key2", 200),
            TestTransaction::simple("key3", 300),
            TestTransaction::simple("key4", 400),
        ];

        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(2, Some(650));

        let result = executor.execute_transactions_parallel((0, GasMode::UseValue), transactions);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    // ========== Block Prologue/Epilogue Tests ==========

    // Extended transaction type to support prologue/epilogue
    #[derive(Debug, Clone)]
    struct BlockTransaction {
        key: String,
        value: u64,
        is_prologue: bool,
        is_epilogue: bool,
        // For epilogue: keys to read (simulating gas fee collection)
        read_keys: Vec<String>,
    }

    impl BlockTransaction {
        fn user_txn(key: &str, value: u64) -> Self {
            Self {
                key: key.to_string(),
                value,
                is_prologue: false,
                is_epilogue: false,
                read_keys: vec![],
            }
        }

        fn prologue(value: u64) -> Self {
            Self {
                key: "prologue".to_string(),
                value,
                is_prologue: true,
                is_epilogue: false,
                read_keys: vec![],
            }
        }

        fn epilogue(read_keys: Vec<String>) -> Self {
            Self {
                key: "epilogue".to_string(),
                value: 0,
                is_prologue: false,
                is_epilogue: true,
                read_keys,
            }
        }
    }

    impl Transaction for BlockTransaction {
        type Key = String;
        type Value = u64;

        fn is_block_prologue(&self) -> bool {
            self.is_prologue
        }

        fn is_block_epilogue(&self) -> bool {
            self.is_epilogue
        }
    }

    #[derive(Debug, Clone)]
    struct BlockOutput {
        writes: HashMap<String, u64>,
        gas: u64,
        // Track which keys were successfully read (for epilogue verification)
        reads: Vec<String>,
    }

    impl TransactionOutput for BlockOutput {
        type T = BlockTransaction;

        fn get_writes(&self) -> Vec<(String, u64)> {
            self.writes.iter().map(|(k, v)| (k.clone(), *v)).collect()
        }

        fn gas_used(&self) -> u64 {
            self.gas
        }

        fn skip_output() -> Self {
            Self {
                writes: HashMap::new(),
                gas: 0,
                reads: vec![],
            }
        }
    }

    struct BlockExecutor {
        initial_value: u64,
    }

    impl ExecutorTask for BlockExecutor {
        type T = BlockTransaction;
        type Output = BlockOutput;
        type Error = ();
        type Argument = u64;

        fn init(initial_value: Self::Argument) -> Self {
            Self { initial_value }
        }

        fn execute_transaction(
            &self,
            view: &MVHashMapView<String, u64>,
            txn: &Self::T,
        ) -> ExecutionStatus<Self::Output, ()> {
            let mut writes = HashMap::new();
            let mut reads = vec![];

            if txn.is_epilogue {
                // Epilogue: try to read from specified keys
                for key in &txn.read_keys {
                    if let Some(_value) = view.read(key) {
                        reads.push(key.clone());
                        // Epilogue doesn't write, just reads
                    }
                }
                ExecutionStatus::Success(BlockOutput {
                    writes,
                    gas: 0,
                    reads,
                })
            } else {
                // Regular transaction or prologue
                let current_value = view
                    .read(&txn.key)
                    .map(|v| *v)
                    .unwrap_or(self.initial_value);
                let new_value = current_value + txn.value;
                writes.insert(txn.key.clone(), new_value);

                ExecutionStatus::Success(BlockOutput {
                    writes,
                    gas: txn.value,
                    reads,
                })
            }
        }
    }

    #[test]
    fn test_epilogue_sees_only_within_gas_limit() {
        // This test verifies epilogue only sees transactions within gas limit

        // Setup: prologue + 4 user txns + epilogue
        // Gas limit: 1000
        // Txn 0 (prologue): 100 gas
        // Txn 1: 200 gas (total: 300)
        // Txn 2: 300 gas (total: 600)
        // Txn 3: 300 gas (total: 900) <- Last valid txn
        // Txn 4: 200 gas (total: 1100) <- EXCEEDS LIMIT
        // Txn 5 (epilogue): should only see txns 0-3
        let transactions = vec![
            BlockTransaction::prologue(100),
            BlockTransaction::user_txn("user1", 200),
            BlockTransaction::user_txn("user2", 300),
            BlockTransaction::user_txn("user3", 300),
            BlockTransaction::user_txn("user4", 200), // This exceeds limit
            BlockTransaction::epilogue(vec![
                "prologue".to_string(),
                "user1".to_string(),
                "user2".to_string(),
                "user3".to_string(),
                "user4".to_string(), // Should NOT be readable
            ]),
        ];

        let executor: ParallelTransactionExecutor<BlockTransaction, BlockExecutor> =
            ParallelTransactionExecutor::new(2, Some(1000));

        let result = executor.execute_transactions_parallel(0, transactions);
        assert!(result.is_ok(), "Execution should succeed");

        let outputs = result.unwrap();

        // should have prologue + user1 + user2 + user3 + epilogue = 5
        assert_eq!(
            outputs.len(),
            5,
            "Should have 5 outputs (prologue + 3 user txns + epilogue)"
        );

        // Verify total gas (excluding epilogue's 0 gas)
        let total_gas: u64 = outputs.iter().map(|o| o.gas).sum();
        assert_eq!(total_gas, 900, "Total gas should be 900 (100+200+300+300)");

        // Verify epilogue (last output)
        let epilogue_output = outputs.last().unwrap();
        assert_eq!(epilogue_output.gas, 0, "Epilogue should have 0 gas");

        // Key assertion: Epilogue sees user3 (within limit) but not user4 (exceeds)
        assert_eq!(
            epilogue_output.reads.len(),
            4,
            "Epilogue reads 4 keys (prologue, user1, user2, user3)"
        );
        assert!(
            epilogue_output.reads.contains(&"user3".to_string()),
            "Epilogue should see user3 (within gas limit)"
        );
        assert!(
            !epilogue_output.reads.contains(&"user4".to_string()),
            "Epilogue should NOT see user4 (exceeds gas limit)"
        );
    }

    #[test]
    fn test_epilogue_all_transactions_within_limit() {
        // All transactions fit within gas limit
        let transactions = vec![
            BlockTransaction::prologue(100),
            BlockTransaction::user_txn("user1", 100),
            BlockTransaction::user_txn("user2", 100),
            BlockTransaction::epilogue(vec![
                "prologue".to_string(),
                "user1".to_string(),
                "user2".to_string(),
            ]),
        ];

        let executor: ParallelTransactionExecutor<BlockTransaction, BlockExecutor> =
            ParallelTransactionExecutor::new(2, Some(500));

        let result = executor.execute_transactions_parallel(0, transactions);
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 4, "All transactions should execute");

        let total_gas: u64 = outputs.iter().map(|o| o.gas).sum();
        assert_eq!(total_gas, 300, "Total gas should be 300");

        let epilogue_output = outputs.last().unwrap();
        assert_eq!(
            epilogue_output.reads.len(),
            3,
            "Epilogue should see all 3 transactions"
        );
    }

    #[test]
    fn test_epilogue_first_user_txn_exceeds() {
        // Test when first user transaction exceeds gas limit
        let transactions = vec![
            BlockTransaction::prologue(100),
            BlockTransaction::user_txn("user1", 1000), // Exceeds limit
            BlockTransaction::epilogue(vec!["prologue".to_string(), "user1".to_string()]),
        ];

        let executor: ParallelTransactionExecutor<BlockTransaction, BlockExecutor> =
            ParallelTransactionExecutor::new(2, Some(500));

        let result = executor.execute_transactions_parallel(0, transactions);
        assert!(result.is_ok());

        let outputs = result.unwrap();

        // should have prologue + epilogue
        assert_eq!(outputs.len(), 2, "Should have prologue and epilogue");
        assert_eq!(outputs[0].gas, 100, "First should be prologue");
        assert_eq!(outputs[1].gas, 0, "Second should be epilogue");

        // Epilogue should see prologue but not user1
        let epilogue_output = &outputs[1];
        assert!(
            epilogue_output.reads.contains(&"prologue".to_string()),
            "Epilogue should see prologue"
        );
        assert!(
            !epilogue_output.reads.contains(&"user1".to_string()),
            "Epilogue should NOT see user1 (exceeds gas limit)"
        );
    }

    #[test]
    fn test_epilogue_without_gas_limit() {
        // No gas limit - epilogue should see all transactions
        let transactions = vec![
            BlockTransaction::prologue(100),
            BlockTransaction::user_txn("user1", 500),
            BlockTransaction::user_txn("user2", 500),
            BlockTransaction::user_txn("user3", 500),
            BlockTransaction::epilogue(vec![
                "prologue".to_string(),
                "user1".to_string(),
                "user2".to_string(),
                "user3".to_string(),
            ]),
        ];

        let executor: ParallelTransactionExecutor<BlockTransaction, BlockExecutor> =
            ParallelTransactionExecutor::new(2, None);

        let result = executor.execute_transactions_parallel(0, transactions);
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 5, "All transactions should execute");

        let epilogue_output = outputs.last().unwrap();
        assert_eq!(
            epilogue_output.reads.len(),
            4,
            "Epilogue should see all transactions when no gas limit"
        );
    }

    #[test]
    fn test_epilogue_exact_gas_boundary() {
        // Test exact gas boundary condition
        // prologue(100) + user1(400) + user2(500) = 1000 (exact limit)
        // user3(1) would make it 1001, so excluded
        let transactions = vec![
            BlockTransaction::prologue(100),
            BlockTransaction::user_txn("user1", 400),
            BlockTransaction::user_txn("user2", 500),
            BlockTransaction::user_txn("user3", 1), // Should be excluded
            BlockTransaction::epilogue(vec![
                "prologue".to_string(),
                "user1".to_string(),
                "user2".to_string(),
                "user3".to_string(),
            ]),
        ];

        let executor: ParallelTransactionExecutor<BlockTransaction, BlockExecutor> =
            ParallelTransactionExecutor::new(2, Some(1000));

        let result = executor.execute_transactions_parallel(0, transactions);
        assert!(result.is_ok());

        let outputs = result.unwrap();
        // prologue + user1 + user2 + epilogue = 4
        assert_eq!(outputs.len(), 4, "Should have 4 outputs");

        let total_gas: u64 = outputs.iter().map(|o| o.gas).sum();
        assert_eq!(total_gas, 1000, "Total gas should be exactly 1000");

        let epilogue_output = outputs.last().unwrap();
        assert_eq!(
            epilogue_output.reads.len(),
            3,
            "Epilogue should see 3 transactions"
        );
        // Key assertion: epilogue should see user2 (within limit) but not user3 (exceeds)
        assert!(
            epilogue_output.reads.contains(&"user2".to_string()),
            "Epilogue should see user2 (within gas limit)"
        );
        assert!(
            !epilogue_output.reads.contains(&"user3".to_string()),
            "Epilogue should NOT see user3 (exceeds gas limit)"
        );
    }

    #[test]
    fn test_block_without_epilogue() {
        // Test block without epilogue (normal case)
        let transactions = vec![
            BlockTransaction::prologue(100),
            BlockTransaction::user_txn("user1", 200),
            BlockTransaction::user_txn("user2", 300),
        ];

        let executor: ParallelTransactionExecutor<BlockTransaction, BlockExecutor> =
            ParallelTransactionExecutor::new(2, Some(1000));

        let result = executor.execute_transactions_parallel(0, transactions);
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 3, "All transactions should execute");

        let total_gas: u64 = outputs.iter().map(|o| o.gas).sum();
        assert_eq!(total_gas, 600);
    }

    #[test]
    fn test_epilogue_with_zero_gas_transactions() {
        // Test epilogue with mix of zero and non-zero gas transactions
        let transactions = vec![
            BlockTransaction::prologue(100),
            BlockTransaction::user_txn("user1", 0), // Zero gas
            BlockTransaction::user_txn("user2", 400),
            BlockTransaction::user_txn("user3", 0), // Zero gas
            BlockTransaction::user_txn("user4", 600), // Exceeds limit
            BlockTransaction::epilogue(vec![
                "prologue".to_string(),
                "user1".to_string(),
                "user2".to_string(),
                "user3".to_string(),
                "user4".to_string(),
            ]),
        ];

        let executor: ParallelTransactionExecutor<BlockTransaction, BlockExecutor> =
            ParallelTransactionExecutor::new(2, Some(1000));

        let result = executor.execute_transactions_parallel(0, transactions);
        assert!(result.is_ok());

        let outputs = result.unwrap();
        // prologue(100) + user1(0) + user2(400) + user3(0) = 500 < 1000
        // user4(600) would make it 1100 > 1000, excluded
        // prologue + user1 + user2 + user3 + epilogue = 5
        assert_eq!(outputs.len(), 5, "Should have 5 outputs");

        let epilogue_output = outputs.last().unwrap();
        assert_eq!(epilogue_output.gas, 0, "Epilogue has 0 gas");

        // Key assertion: epilogue should see user1-3 but not user4
        assert_eq!(
            epilogue_output.reads.len(),
            4,
            "Epilogue should see 4 transactions"
        );
        assert!(
            epilogue_output.reads.contains(&"user3".to_string()),
            "Epilogue should see user3 (within gas limit)"
        );
        assert!(
            !epilogue_output.reads.contains(&"user4".to_string()),
            "Epilogue should NOT see user4 (exceeds gas limit)"
        );
    }

    #[test]
    fn test_result_only_contains_executed_and_epilogue() {
        // This test verifies the critical behavior:
        // 1. Only executed transactions (within gas limit) are returned
        // 2. Epilogue is ALWAYS returned even when user txns are excluded
        // 3. Out-of-gas transactions are completely excluded from results

        // Setup: prologue + 5 user txns + epilogue, gas limit = 700
        // prologue: 100 gas (total: 100) ✓
        // user1: 200 gas (total: 300) ✓
        // user2: 300 gas (total: 600) ✓
        // user3: 200 gas (total: 800) ✗ OUT OF GAS
        // user4: 100 gas (total: 900) ✗ OUT OF GAS
        // user5: 50 gas (total: 950) ✗ OUT OF GAS
        // epilogue: always executes ✓
        let transactions = vec![
            BlockTransaction::prologue(100),
            BlockTransaction::user_txn("user1", 200),
            BlockTransaction::user_txn("user2", 300),
            BlockTransaction::user_txn("user3", 200), // Out of gas
            BlockTransaction::user_txn("user4", 100), // Out of gas
            BlockTransaction::user_txn("user5", 50),  // Out of gas
            BlockTransaction::epilogue(vec![
                "prologue".to_string(),
                "user1".to_string(),
                "user2".to_string(),
                "user3".to_string(),
                "user4".to_string(),
                "user5".to_string(),
            ]),
        ];

        let executor: ParallelTransactionExecutor<BlockTransaction, BlockExecutor> =
            ParallelTransactionExecutor::new(2, Some(700));

        let result = executor.execute_transactions_parallel(0, transactions);
        assert!(result.is_ok(), "Execution should succeed");

        let outputs = result.unwrap();

        // KEY ASSERTION: Only executed txns + epilogue
        // Expected: prologue + user1 + user2 + epilogue = 4 outputs
        assert_eq!(outputs.len(), 4, "Should return 4 outputs: prologue, user1, user2, epilogue. Out-of-gas user3/4/5 excluded");

        // Verify the returned transactions are correct
        assert!(
            outputs[0].writes.contains_key("prologue"),
            "First should be prologue"
        );
        assert!(
            outputs[1].writes.contains_key("user1"),
            "Second should be user1"
        );
        assert!(
            outputs[2].writes.contains_key("user2"),
            "Third should be user2"
        );
        assert_eq!(outputs[3].gas, 0, "Last should be epilogue (0 gas)");

        // Verify total gas (excluding epilogue's 0 gas)
        let total_gas: u64 = outputs.iter().map(|o| o.gas).sum();
        assert_eq!(
            total_gas, 600,
            "Total gas should be 600 (100+200+300), not including out-of-gas txns"
        );

        // Verify epilogue behavior
        let epilogue_output = outputs.last().unwrap();
        assert_eq!(
            epilogue_output.reads.len(),
            3,
            "Epilogue should only see 3 txns (prologue, user1, user2)"
        );
        assert!(
            epilogue_output.reads.contains(&"user2".to_string()),
            "Epilogue should see user2 (last within gas limit)"
        );
        assert!(
            !epilogue_output.reads.contains(&"user3".to_string()),
            "Epilogue should NOT see user3 (first to exceed gas limit)"
        );
    }

    #[derive(Debug, Clone)]
    struct DelayedTxn {
        id: DelayedFieldID,
        value: u128,
        emit_change: bool,
        fail_first: bool,
        attempts: Arc<AtomicUsize>,
    }

    impl Transaction for DelayedTxn {
        type Key = u64;
        type Value = u64;
    }

    #[derive(Debug, Clone)]
    struct DelayedOutput {
        gas: u64,
        changes: Vec<(DelayedFieldID, DelayedChange<DelayedFieldID>)>,
    }

    impl TransactionOutput for DelayedOutput {
        type T = DelayedTxn;

        fn get_writes(&self) -> Vec<(u64, u64)> {
            Vec::new()
        }

        fn gas_used(&self) -> u64 {
            self.gas
        }

        fn skip_output() -> Self {
            Self {
                gas: 0,
                changes: Vec::new(),
            }
        }

        fn delayed_field_change_set(&self) -> Vec<(DelayedFieldID, DelayedChange<DelayedFieldID>)> {
            self.changes.clone()
        }
    }

    struct DelayedExecutor;

    impl ExecutorTask for DelayedExecutor {
        type T = DelayedTxn;
        type Output = DelayedOutput;
        type Error = ();
        type Argument = ();

        fn init(_args: Self::Argument) -> Self {
            Self
        }

        fn execute_transaction(
            &self,
            view: &MVHashMapView<u64, u64>,
            txn: &Self::T,
        ) -> ExecutionStatus<Self::Output, ()> {
            let attempt = txn.attempts.fetch_add(1, Ordering::SeqCst);
            if txn.fail_first && attempt == 0 {
                view.capture_delayed_field_read_error(&PanicOr::Or(
                    DelayedFieldsSpeculativeError::InconsistentRead,
                ));
            }
            let changes = if txn.emit_change {
                vec![(
                    txn.id,
                    DelayedChange::Create(DelayedFieldValue::Aggregator(txn.value)),
                )]
            } else {
                Vec::new()
            };
            ExecutionStatus::Success(DelayedOutput { gas: 0, changes })
        }
    }

    #[test]
    fn test_delayed_field_reexecution_and_commit() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let txns = vec![
            DelayedTxn {
                id: DelayedFieldID::new_with_width(1000, 8),
                value: 10,
                emit_change: true,
                fail_first: true,
                attempts: attempts.clone(),
            },
            DelayedTxn {
                id: DelayedFieldID::new_with_width(1001, 8),
                value: 20,
                emit_change: true,
                fail_first: false,
                attempts: Arc::new(AtomicUsize::new(0)),
            },
        ];

        let executor: ParallelTransactionExecutor<DelayedTxn, DelayedExecutor> =
            ParallelTransactionExecutor::new(2, None).with_delayed_fields(true);
        let (_outputs, delayed_fields) = executor
            .execute_transactions_parallel_with_delayed_fields((), txns)
            .unwrap();

        assert!(
            attempts.load(Ordering::SeqCst) > 1,
            "expected at least one re-execution for speculative failure"
        );

        let id0 = DelayedFieldID::new_with_width(1000, 8);
        let id1 = DelayedFieldID::new_with_width(1001, 8);
        let v0 = delayed_fields
            .read_latest_predicted_value(&id0, 2, ReadPosition::AfterCurrentTxn)
            .unwrap();
        let v1 = delayed_fields
            .read_latest_predicted_value(&id1, 2, ReadPosition::AfterCurrentTxn)
            .unwrap();

        assert_eq!(v0, DelayedFieldValue::Aggregator(10));
        assert_eq!(v1, DelayedFieldValue::Aggregator(20));
    }

    #[test]
    fn test_delayed_field_duplicate_create_is_fatal() {
        let id = DelayedFieldID::new_with_width(2000, 8);
        let txns = vec![
            DelayedTxn {
                id,
                value: 10,
                emit_change: true,
                fail_first: false,
                attempts: Arc::new(AtomicUsize::new(0)),
            },
            DelayedTxn {
                id,
                value: 20,
                emit_change: true,
                fail_first: false,
                attempts: Arc::new(AtomicUsize::new(0)),
            },
        ];

        let executor: ParallelTransactionExecutor<DelayedTxn, DelayedExecutor> =
            ParallelTransactionExecutor::new(2, None).with_delayed_fields(true);
        let result = executor.execute_transactions_parallel_with_delayed_fields((), txns);

        assert!(matches!(result, Err(Error::InvariantViolation)));
    }

    #[derive(Debug, Clone)]
    struct EpilogueTxn {
        epilogue: bool,
    }

    impl Transaction for EpilogueTxn {
        type Key = u64;
        type Value = u64;

        fn is_block_epilogue(&self) -> bool {
            self.epilogue
        }
    }

    #[derive(Debug, Clone)]
    struct EpilogueOutput;

    impl TransactionOutput for EpilogueOutput {
        type T = EpilogueTxn;

        fn get_writes(&self) -> Vec<(u64, u64)> {
            Vec::new()
        }

        fn gas_used(&self) -> u64 {
            0
        }

        fn skip_output() -> Self {
            Self
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum EpilogueError {
        EpilogueFail,
    }

    struct EpilogueExecutor;

    impl ExecutorTask for EpilogueExecutor {
        type T = EpilogueTxn;
        type Output = EpilogueOutput;
        type Error = EpilogueError;
        type Argument = ();

        fn init(_args: Self::Argument) -> Self {
            Self
        }

        fn execute_transaction(
            &self,
            _view: &MVHashMapView<u64, u64>,
            txn: &Self::T,
        ) -> ExecutionStatus<Self::Output, Self::Error> {
            if txn.is_block_epilogue() {
                ExecutionStatus::Abort(EpilogueError::EpilogueFail)
            } else {
                ExecutionStatus::Success(EpilogueOutput)
            }
        }
    }

    #[test]
    fn test_epilogue_abort_returns_error() {
        let txns = vec![
            EpilogueTxn { epilogue: false },
            EpilogueTxn { epilogue: true },
        ];

        let executor: ParallelTransactionExecutor<EpilogueTxn, EpilogueExecutor> =
            ParallelTransactionExecutor::new(2, None);
        let result = executor.execute_transactions_parallel((), txns);

        assert!(matches!(
            result,
            Err(Error::UserError(EpilogueError::EpilogueFail))
        ));
    }

    #[derive(Debug, Clone)]
    struct GasDelayedTxn {
        id: DelayedFieldID,
        value: u128,
        gas: u64,
    }

    impl Transaction for GasDelayedTxn {
        type Key = u64;
        type Value = u64;
    }

    #[derive(Debug, Clone)]
    struct GasDelayedOutput {
        gas: u64,
        changes: Vec<(DelayedFieldID, DelayedChange<DelayedFieldID>)>,
    }

    impl TransactionOutput for GasDelayedOutput {
        type T = GasDelayedTxn;

        fn get_writes(&self) -> Vec<(u64, u64)> {
            Vec::new()
        }

        fn gas_used(&self) -> u64 {
            self.gas
        }

        fn skip_output() -> Self {
            Self {
                gas: 0,
                changes: Vec::new(),
            }
        }

        fn delayed_field_change_set(&self) -> Vec<(DelayedFieldID, DelayedChange<DelayedFieldID>)> {
            self.changes.clone()
        }
    }

    struct GasDelayedExecutor;

    impl ExecutorTask for GasDelayedExecutor {
        type T = GasDelayedTxn;
        type Output = GasDelayedOutput;
        type Error = ();
        type Argument = ();

        fn init(_args: Self::Argument) -> Self {
            Self
        }

        fn execute_transaction(
            &self,
            _view: &MVHashMapView<u64, u64>,
            txn: &Self::T,
        ) -> ExecutionStatus<Self::Output, ()> {
            let changes = vec![(
                txn.id,
                DelayedChange::Create(DelayedFieldValue::Aggregator(txn.value)),
            )];
            ExecutionStatus::Success(GasDelayedOutput {
                gas: txn.gas,
                changes,
            })
        }
    }

    #[test]
    fn test_delayed_fields_beyond_gas_limit_are_discarded() {
        let txns = vec![
            GasDelayedTxn {
                id: DelayedFieldID::new_with_width(3000, 8),
                value: 10,
                gas: 10,
            },
            GasDelayedTxn {
                id: DelayedFieldID::new_with_width(3001, 8),
                value: 20,
                gas: 10,
            },
        ];

        let executor: ParallelTransactionExecutor<GasDelayedTxn, GasDelayedExecutor> =
            ParallelTransactionExecutor::new(2, Some(10)).with_delayed_fields(true);
        let (_outputs, delayed_fields) = executor
            .execute_transactions_parallel_with_delayed_fields((), txns)
            .unwrap();

        let skipped_id = DelayedFieldID::new_with_width(3001, 8);
        let res = delayed_fields.read_latest_predicted_value(
            &skipped_id,
            2,
            ReadPosition::AfterCurrentTxn,
        );

        assert!(matches!(res, Err(MVDelayedFieldsError::NotFound)));
    }
}
