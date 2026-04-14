// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use rand::{rngs::StdRng, RngExt, SeedableRng};
use starcoin_parallel_executor::errors::Error;
use starcoin_parallel_executor::executor::ParallelTransactionExecutor;
use starcoin_parallel_executor::task::{
    ExecutionStatus, ExecutorTask, Transaction, TransactionOutput,
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
enum TxnKind {
    Normal,
    Abort,
    SkipRest,
}

#[derive(Clone, Debug)]
struct MockTxn {
    reads: Vec<u64>,
    writes: Vec<u64>,
    gas: u64,
    salt: u64,
    kind: TxnKind,
}

impl Transaction for MockTxn {
    type Key = u64;
    type Value = u64;
}

#[derive(Clone, Debug)]
struct MockOutput {
    writes: Vec<(u64, u64)>,
    gas: u64,
}

impl TransactionOutput for MockOutput {
    type T = MockTxn;

    fn get_writes(&self) -> Vec<(u64, u64)> {
        self.writes.clone()
    }

    fn gas_used(&self) -> u64 {
        self.gas
    }

    fn skip_output() -> Self {
        Self {
            writes: Vec::new(),
            gas: 0,
        }
    }
}

struct MockExecutor;

impl ExecutorTask for MockExecutor {
    type T = MockTxn;
    type Output = MockOutput;
    type Error = ();
    type Argument = ();

    fn init(_args: Self::Argument) -> Self {
        Self
    }

    fn execute_transaction(
        &self,
        view: &starcoin_parallel_executor::executor::MVHashMapView<u64, u64>,
        txn: &Self::T,
    ) -> ExecutionStatus<Self::Output, ()> {
        if matches!(txn.kind, TxnKind::Abort) {
            return ExecutionStatus::Abort(());
        }

        let mut sum = 0u64;
        for key in &txn.reads {
            sum = sum.wrapping_add(view.read(key).map(|v| *v).unwrap_or(0));
        }

        let mut writes = Vec::with_capacity(txn.writes.len());
        for key in &txn.writes {
            let value = sum.wrapping_add(txn.salt).wrapping_add(*key);
            writes.push((*key, value));
        }

        let output = MockOutput {
            writes,
            gas: txn.gas,
        };

        if matches!(txn.kind, TxnKind::SkipRest) {
            ExecutionStatus::SkipRest(output)
        } else {
            ExecutionStatus::Success(output)
        }
    }
}

fn gen_transactions(
    seed: u64,
    universe_size: usize,
    txn_count: usize,
    reads_per_txn: std::ops::RangeInclusive<usize>,
    writes_per_txn: std::ops::RangeInclusive<usize>,
    gas_range: std::ops::RangeInclusive<u64>,
) -> Vec<MockTxn> {
    let mut rng = StdRng::seed_from_u64(seed);
    let keys: Vec<u64> = (0..universe_size as u64).collect();
    let mut txns = Vec::with_capacity(txn_count);

    for idx in 0..txn_count {
        let read_cnt = rng.random_range(reads_per_txn.clone());
        let write_cnt = rng.random_range(writes_per_txn.clone());
        let mut reads = HashSet::new();
        let mut writes = HashSet::new();

        while reads.len() < read_cnt {
            reads.insert(keys[rng.random_range(0..keys.len())]);
        }
        while writes.len() < write_cnt {
            writes.insert(keys[rng.random_range(0..keys.len())]);
        }

        txns.push(MockTxn {
            reads: reads.into_iter().collect(),
            writes: writes.into_iter().collect(),
            gas: rng.random_range(gas_range.clone()),
            salt: idx as u64,
            kind: TxnKind::Normal,
        });
    }

    txns
}

fn execute_sequential(txns: &[MockTxn], gas_limit: Option<u64>) -> (HashMap<u64, u64>, usize) {
    let mut state = HashMap::new();
    let mut gas_used = 0u64;
    let mut executed = 0usize;

    for txn in txns {
        if let Some(limit) = gas_limit {
            if gas_used + txn.gas > limit {
                break;
            }
        }
        gas_used += txn.gas;
        executed += 1;

        let mut sum = 0u64;
        for key in &txn.reads {
            sum = sum.wrapping_add(*state.get(key).unwrap_or(&0));
        }
        for key in &txn.writes {
            let value = sum.wrapping_add(txn.salt).wrapping_add(*key);
            state.insert(*key, value);
        }
    }

    (state, executed)
}

fn assert_parallel_matches_sequential(
    txns: Vec<MockTxn>,
    gas_limit: Option<u64>,
    concurrency_level: usize,
) {
    let executor: ParallelTransactionExecutor<MockTxn, MockExecutor> =
        ParallelTransactionExecutor::new(concurrency_level, gas_limit);

    let (mut outputs, _delayed_fields) = executor
        .execute_transactions_parallel_with_delayed_fields((), txns.clone())
        .expect("parallel execution should succeed");
    outputs.sort_by_key(|(idx, _)| *idx);

    let (expected_state, expected_count) = execute_sequential(&txns, gas_limit);
    assert_eq!(outputs.len(), expected_count);
    for (pos, (idx, _)) in outputs.iter().enumerate() {
        assert_eq!(
            *idx, pos,
            "gas-limited outputs should be a contiguous prefix"
        );
    }

    let mut state = HashMap::new();
    for (_, output) in outputs {
        for (k, v) in output.get_writes() {
            state.insert(k, v);
        }
    }

    assert_eq!(state, expected_state);
}

#[test]
fn resource_combinatorial_no_gas_limit() {
    for seed in [1u64, 2, 3] {
        let txns = gen_transactions(seed, 30, 200, 1..=3, 1..=2, 1..=4);
        assert_parallel_matches_sequential(txns, None, num_cpus::get().max(2));
    }
}

#[test]
fn resource_combinatorial_with_gas_limit() {
    for seed in [10u64, 11, 12, 13, 14, 15] {
        let txns = gen_transactions(seed, 40, 200, 1..=3, 1..=3, 1..=5);
        assert_parallel_matches_sequential(txns.clone(), Some(100), num_cpus::get().max(2));
        assert_parallel_matches_sequential(txns, Some(0), num_cpus::get().max(2));
    }
}

#[test]
fn resource_combinatorial_group_like_writes() {
    let mut rng = StdRng::seed_from_u64(99);
    let mut txns = Vec::new();

    for idx in 0..100 {
        let group_id = rng.random_range(0..20u64);
        let base = group_id * 10;
        let writes = vec![base, base + 1, base + 2];
        let reads = vec![base];
        txns.push(MockTxn {
            reads,
            writes,
            gas: rng.random_range(1..=3),
            salt: idx as u64,
            kind: TxnKind::Normal,
        });
    }

    assert_parallel_matches_sequential(txns, None, num_cpus::get().max(2));
}

#[test]
fn resource_gas_limit_small_high_concurrency() {
    let txns = gen_transactions(77, 20, 30, 1..=3, 1..=3, 1..=5);
    assert_parallel_matches_sequential(txns, Some(60), num_cpus::get().max(2));
}

#[test]
fn resource_abort_returns_user_error() {
    let mut txns = gen_transactions(42, 10, 10, 1..=2, 1..=1, 1..=2);
    txns[5].kind = TxnKind::Abort;

    let executor: ParallelTransactionExecutor<MockTxn, MockExecutor> =
        ParallelTransactionExecutor::new(num_cpus::get().max(2), None);

    let result = executor.execute_transactions_parallel((), txns);
    assert!(matches!(result, Err(Error::UserError(()))));
}

#[test]
fn resource_skip_rest_returns_block_restart() {
    let mut txns = gen_transactions(43, 10, 10, 1..=2, 1..=1, 1..=2);
    txns[4].kind = TxnKind::SkipRest;

    let executor: ParallelTransactionExecutor<MockTxn, MockExecutor> =
        ParallelTransactionExecutor::new(num_cpus::get().max(2), None);

    let result = executor.execute_transactions_parallel((), txns);
    assert!(matches!(result, Err(Error::BlockRestart)));
}
