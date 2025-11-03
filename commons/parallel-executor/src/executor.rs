// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::Error::BlockRestart;
use crate::{
    errors::*,
    scheduler::{Scheduler, SchedulerTask, TaskGuard, TxnIndex, Version},
    task::{ExecutionStatus, ExecutorTask, Transaction, TransactionOutput},
    txn_last_input_output::{ReadDescriptor, TxnLastInputOutput},
};
use num_cpus;
use once_cell::sync::Lazy;
use starcoin_infallible::Mutex;
use starcoin_logger::prelude::error;
use starcoin_mvhashmap::MVHashMap;
use std::{collections::HashSet, hash::Hash, marker::PhantomData, sync::Arc, thread::spawn};

static RAYON_EXEC_POOL: Lazy<rayon::ThreadPool> = Lazy::new(|| {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .thread_name(|index| format!("parallel_executor_{}", index))
        .build()
        .unwrap()
});

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
}

impl<K: PartialOrd + Send + Clone + Hash + Eq, V: Send + Sync> MVHashMapView<'_, K, V> {
    /// Drains the captured reads.
    pub fn take_reads(&self) -> Vec<ReadDescriptor<K>> {
        let mut reads = self.captured_reads.lock();
        std::mem::take(&mut reads)
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
}
pub struct ParallelTransactionExecutor<T: Transaction, E: ExecutorTask> {
    // number of active concurrent tasks, corresponding to the maximum number of rayon
    // threads that may be concurrently participating in parallel execution.
    concurrency_level: usize,
    phantom: PhantomData<(T, E)>,
    gas_limit: Option<u64>,
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
        Self {
            concurrency_level,
            phantom: PhantomData,
            gas_limit,
        }
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
    ) -> SchedulerTask<'a> {
        let (idx_to_execute, incarnation) = version;
        let txn = &signature_verified_block[idx_to_execute];

        let state_view = MVHashMapView {
            versioned_map: versioned_data_cache,
            txn_idx: idx_to_execute,
            scheduler,
            captured_reads: Mutex::new(Vec::new()),
        };

        // VM execution.
        let execute_result = executor.execute_transaction(&state_view, txn);
        let mut prev_write_set: HashSet<T::Key> = last_input_output.write_set(idx_to_execute);

        // For tracking whether the recent execution wrote outside of the previous write set.
        let mut writes_outside = false;
        let mut apply_writes = |output: &<E as ExecutorTask>::Output| {
            let write_version = (idx_to_execute, incarnation);
            for (k, v) in output.get_writes().into_iter() {
                if !prev_write_set.remove(&k) {
                    writes_outside = true
                }
                versioned_data_cache.write(&k, write_version, v);
            }
        };

        let result = match execute_result {
            // These statuses are the results of speculative execution, so even for
            // SkipRest (skip the rest of transactions) and Abort (abort execution with
            // user defined error), no immediate action is taken. Instead the statuses
            // are recorded and (final statuses) are analyzed when the block is executed.
            ExecutionStatus::Success(output) => {
                // Apply the writes to the versioned_data_cache.
                apply_writes(&output);
                ExecutionStatus::Success(output)
            }
            ExecutionStatus::SkipRest(output) => {
                // Apply the writes and record status indicating skip.
                apply_writes(&output);
                ExecutionStatus::SkipRest(output)
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

        last_input_output.record(idx_to_execute, state_view.take_reads(), result);
        scheduler.finish_execution(idx_to_execute, incarnation, writes_outside, guard)
    }

    fn validate<'a>(
        &self,
        version_to_validate: Version,
        guard: TaskGuard<'a>,
        last_input_output: &TxnLastInputOutput<
            <T as Transaction>::Key,
            <E as ExecutorTask>::Output,
            <E as ExecutorTask>::Error,
        >,
        versioned_data_cache: &MVHashMap<<T as Transaction>::Key, <T as Transaction>::Value>,
        scheduler: &'a Scheduler,
    ) -> SchedulerTask<'a> {
        let (idx_to_validate, incarnation) = version_to_validate;
        let read_set = last_input_output
            .read_set(idx_to_validate)
            .expect("Prior read-set must be recorded");

        let valid = read_set.iter().all(|r| {
            match versioned_data_cache.read(r.path(), idx_to_validate) {
                Ok((version, _)) => r.validate_version(version),
                Err(Some(_)) => false, // Dependency implies a validation failure.
                Err(None) => r.validate_storage(),
            }
        });

        let aborted = !valid && scheduler.try_abort(idx_to_validate, incarnation);

        if aborted {
            // Not valid and successfully aborted, mark the latest write-set as estimates.
            for k in &last_input_output.write_set(idx_to_validate) {
                versioned_data_cache.mark_estimate(k, idx_to_validate);
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
    ) {
        // Make executor for each task. TODO: fast concurrent executor.
        let executor = E::init(*executor_arguments);

        let mut scheduler_task = SchedulerTask::NoTask;
        loop {
            scheduler_task = match scheduler_task {
                SchedulerTask::ValidationTask(version_to_validate, guard) => self.validate(
                    version_to_validate,
                    guard,
                    last_input_output,
                    versioned_data_cache,
                    scheduler,
                ),
                SchedulerTask::ExecutionTask(version_to_execute, None, guard) => self.execute(
                    version_to_execute,
                    guard,
                    block,
                    last_input_output,
                    versioned_data_cache,
                    scheduler,
                    &executor,
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
    ) {
        if block.is_empty() || !block[0].is_block_prologue() {
            return;
        }

        let executor = E::init(*executor_arguments);
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
                    last_input_output,
                    versioned_data_cache,
                    scheduler,
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
    ) {
        // Check if block has epilogue
        if block.is_empty() || !block[block.len() - 1].is_block_epilogue() {
            return;
        }

        // Get the first index that exceeds gas limit
        let first_exceeding = scheduler.first_exceeding_index();
        let epilogue_idx = block.len() - 1;

        // Collect senders from the first n valid transactions (excluding prologue and epilogue)
        let senders = block[..first_exceeding]
            .iter()
            .filter_map(|txn| {
                if txn.is_block_prologue() || txn.is_block_epilogue() {
                    None
                } else {
                    txn.sender()
                }
            })
            .collect::<HashSet<_>>();

        // Clone and update the epilogue transaction with senders
        let mut epilogue_txn = block[epilogue_idx].clone();
        epilogue_txn.update_senders_for_epilogue(&senders);

        // Create a snapshot view that only sees transactions up to first_exceeding
        // This view will read from versioned_data_cache as if it's at index first_exceeding
        let state_view = MVHashMapView {
            versioned_map: versioned_data_cache,
            txn_idx: first_exceeding, // This makes it see state after first_exceeding-1
            scheduler,
            captured_reads: Mutex::new(Vec::new()),
        };

        // Execute block epilogue with the snapshot view
        let executor = E::init(*executor_arguments);
        let execute_result = executor.execute_transaction(&state_view, &epilogue_txn);

        // Apply the writes from epilogue execution
        let apply_writes = |output: &<E as ExecutorTask>::Output| {
            let write_version = (epilogue_idx, 0);
            for (k, v) in output.get_writes().into_iter() {
                versioned_data_cache.write(&k, write_version, v);
            }
        };

        let result = match execute_result {
            ExecutionStatus::Success(output) => {
                apply_writes(&output);
                ExecutionStatus::Success(output)
            }
            _ => unreachable!("block epilogue execution should not fail"),
        };

        // Record the epilogue execution result
        last_input_output.record(epilogue_idx, state_view.take_reads(), result);
    }

    pub fn execute_transactions_parallel(
        &self,
        executor_initial_arguments: E::Argument,
        signature_verified_block: Vec<T>,
    ) -> Result<Vec<E::Output>, E::Error> {
        if signature_verified_block.is_empty() {
            return Ok(vec![]);
        }

        let num_txns = signature_verified_block.len();
        let versioned_data_cache = MVHashMap::new();
        let last_input_output = TxnLastInputOutput::new(num_txns);
        let scheduler = Scheduler::new(num_txns, self.gas_limit);

        // BlockMetadata is always the first txn of block that modifies fundamental info of block
        // other txns depends on this execution result, execute it first to avoid unnecessary contention
        self.execute_block_prologue(
            &executor_initial_arguments,
            &signature_verified_block,
            &last_input_output,
            &versioned_data_cache,
            &scheduler,
        );

        RAYON_EXEC_POOL.scope(|s| {
            for _ in 0..self.concurrency_level {
                s.spawn(|_| {
                    self.work_task_with_scope(
                        &executor_initial_arguments,
                        &signature_verified_block,
                        &last_input_output,
                        &versioned_data_cache,
                        &scheduler,
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
        );

        // Get the actual number of transactions to execute (considering gas limit)
        let first_exceeding = scheduler.first_exceeding_index();
        let num_txns_to_collect = first_exceeding.min(scheduler.num_txn_to_execute());

        // Check if there's a block epilogue
        let has_epilogue = !signature_verified_block.is_empty()
            && signature_verified_block[signature_verified_block.len() - 1].is_block_epilogue();

        // TODO: for large block sizes and many cores, extract outputs in parallel.
        let mut maybe_err = None;
        let mut final_results = Vec::with_capacity(num_txns_to_collect);

        let index = if has_epilogue {
            num_txns_to_collect - 1
        } else {
            num_txns_to_collect
        };
        for idx in 0..index {
            match last_input_output.take_output(idx) {
                ExecutionStatus::Success(t) => final_results.push(t),
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
        if has_epilogue && maybe_err.is_none() {
            let epilogue_idx = signature_verified_block.len() - 1;
            match last_input_output.take_output(epilogue_idx) {
                ExecutionStatus::Success(t) => final_results.push(t),
                ExecutionStatus::SkipRest(_t) => {
                    maybe_err = Some(BlockRestart);
                }
                ExecutionStatus::Abort(err) => {
                    maybe_err = Some(err);
                }
            };
        }

        spawn(move || {
            // Explicit async drops.
            drop(last_input_output);
            drop(signature_verified_block);
            drop(versioned_data_cache);
            drop(scheduler);
        });

        match maybe_err {
            Some(err) => Err(err),
            None => Ok(final_results),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Condvar;

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
        type Sender = String;

        fn sender(&self) -> Option<Self::Sender> {
            Some("test_sender".to_string())
        }
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
        Fixed(u64),   // Fixed gas per transaction
        UseValue,     // Use transaction value as gas
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

        let result = executor.execute_transactions_parallel(
            (initial_value, GasMode::Fixed(100)),
            transactions,
        );
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
    fn test_parallel_independent_transactions() {
        // Create two transactions that operate on different keys (no conflicts)
        let transactions = vec![
            TestTransaction::simple("account_a", 100),
            TestTransaction::simple("account_b", 200),
        ];

        let initial_value = 50;
        let executor: ParallelTransactionExecutor<TestTransaction, TestExecutor> =
            ParallelTransactionExecutor::new(num_cpus::get().max(2), None);

        let result = executor.execute_transactions_parallel(
            (initial_value, GasMode::Fixed(100)),
            transactions,
        );
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
        assert_eq!(result.unwrap().len(), 2, "Only first 2 transactions should execute");
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
        assert_eq!(result.unwrap().len(), 2, "First 2 transactions should execute exactly");
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
        assert_eq!(result.unwrap().len(), 3, "All transactions should execute without limit");
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
        assert_eq!(result.unwrap().len(), 10, "All conflicting transactions should execute");
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
}
