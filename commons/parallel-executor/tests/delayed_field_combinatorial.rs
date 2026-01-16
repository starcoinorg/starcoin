// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_vm_types::delayed_values::delayed_field_id::{DelayedFieldID, ExtractUniqueIndex};
use rand::{rngs::StdRng, Rng, SeedableRng};
use starcoin_aggregator::bounded_math::SignedU128;
use starcoin_aggregator::delayed_change::{DelayedApplyChange, DelayedChange};
use starcoin_aggregator::delta_change_set::DeltaWithMax;
use starcoin_aggregator::types::{DelayedFieldValue, ReadPosition};
use starcoin_mvhashmap::types::MVDelayedFieldsError;
use starcoin_mvhashmap::versioned_delayed_fields::TVersionedDelayedFieldView;
use starcoin_parallel_executor::errors::Error;
use starcoin_parallel_executor::executor::ParallelTransactionExecutor;
use starcoin_parallel_executor::task::{
    ExecutionStatus, ExecutorTask, Transaction, TransactionOutput,
};
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug)]
struct MockTxn {
    gas: u64,
    key: u64,
    op: DelayedOp,
}

#[derive(Clone, Debug)]
enum DelayedOp {
    Create {
        id: DelayedFieldID,
        value: u128,
    },
    Delta {
        id: DelayedFieldID,
        delta: u128,
        max: u128,
    },
}

impl Transaction for MockTxn {
    type Key = u64;
    type Value = u64;
}

#[derive(Clone, Debug)]
struct MockOutput {
    gas: u64,
    writes: Vec<(u64, u64)>,
    changes: Vec<(DelayedFieldID, DelayedChange<DelayedFieldID>)>,
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
            gas: 0,
            writes: Vec::new(),
            changes: Vec::new(),
        }
    }

    fn delayed_field_change_set(&self) -> Vec<(DelayedFieldID, DelayedChange<DelayedFieldID>)> {
        self.changes.clone()
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
        let changes = match &txn.op {
            DelayedOp::Create { id, value } => vec![(
                *id,
                DelayedChange::Create(DelayedFieldValue::Aggregator(*value)),
            )],
            DelayedOp::Delta { id, delta, max } => {
                let _ = view.read(&txn.key);
                vec![(
                    *id,
                    DelayedChange::Apply(DelayedApplyChange::AggregatorDelta {
                        delta: DeltaWithMax::new(SignedU128::Positive(*delta), *max),
                    }),
                )]
            }
        };

        let writes = match &txn.op {
            DelayedOp::Create { .. } => vec![(txn.key, 1)],
            DelayedOp::Delta { .. } => Vec::new(),
        };

        ExecutionStatus::Success(MockOutput {
            gas: txn.gas,
            writes,
            changes,
        })
    }
}

fn gen_transactions(
    seed: u64,
    txn_count: usize,
    id_count: usize,
    gas_range: std::ops::RangeInclusive<u64>,
    max_delta: u128,
) -> (Vec<MockTxn>, BTreeSet<DelayedFieldID>) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut txns = Vec::with_capacity(txn_count);
    let mut all_ids = BTreeSet::new();

    for idx in 0..id_count {
        let id = DelayedFieldID::new_with_width(1000 + idx as u32, 8);
        all_ids.insert(id);
    }
    let ids: Vec<_> = all_ids.iter().copied().collect();

    let create_count = txn_count.min(id_count);
    for id in ids.iter().take(create_count).copied() {
        let gas = rng.random_range(gas_range.clone());
        let value = rng.random_range(1..=1000);
        txns.push(MockTxn {
            gas,
            key: id.extract_unique_index() as u64,
            op: DelayedOp::Create { id, value },
        });
    }

    for _ in create_count..txn_count {
        let gas = rng.random_range(gas_range.clone());
        let id = ids[rng.random_range(0..create_count.max(1))];
        let delta = rng.random_range(1..=max_delta);
        let max = u128::MAX;
        txns.push(MockTxn {
            gas,
            key: id.extract_unique_index() as u64,
            op: DelayedOp::Delta { id, delta, max },
        });
    }

    (txns, all_ids)
}

fn expected_after_sequential(
    txns: &[MockTxn],
    gas_limit: Option<u64>,
) -> (HashMap<DelayedFieldID, u128>, usize) {
    let mut state: HashMap<DelayedFieldID, u128> = HashMap::new();
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

        match txn.op {
            DelayedOp::Create { id, value } => {
                state.insert(id, value);
            }
            DelayedOp::Delta { id, delta, max } => {
                let entry = state
                    .get_mut(&id)
                    .expect("delta should only happen after create");
                let delta = DeltaWithMax::new(SignedU128::Positive(delta), max);
                *entry = delta.apply_to(*entry).expect("delta should fit");
            }
        }
    }

    (state, executed)
}

fn assert_parallel_matches_expected(
    txns: Vec<MockTxn>,
    all_ids: BTreeSet<DelayedFieldID>,
    gas_limit: Option<u64>,
) -> usize {
    let executor: ParallelTransactionExecutor<MockTxn, MockExecutor> =
        ParallelTransactionExecutor::new(num_cpus::get().max(2), gas_limit)
            .with_delayed_fields(true);

    const MAX_RESTARTS: usize = 5;
    let (outputs, delayed_fields, restarts) = {
        let mut restarts = 0usize;
        loop {
            match executor.execute_transactions_parallel_with_delayed_fields((), txns.clone()) {
                Ok(result) => break (result.0, result.1, restarts),
                Err(Error::BlockRestart) => {
                    restarts += 1;
                    if restarts > MAX_RESTARTS {
                        panic!("too many BlockRestart retries: {}", restarts);
                    }
                }
                Err(err) => {
                    panic!("parallel execution should succeed, got {:?}", err);
                }
            }
        }
    };

    let mut outputs = outputs;
    outputs.sort_by_key(|(idx, _)| *idx);
    let executed_parallel = outputs.len();

    let (expected, executed_seq) = expected_after_sequential(&txns, gas_limit);
    assert_eq!(
        executed_parallel, executed_seq,
        "parallel executed count should match sequential"
    );

    for id in all_ids {
        let res = delayed_fields.read_latest_predicted_value(
            &id,
            executed_parallel,
            ReadPosition::AfterCurrentTxn,
        );
        match expected.get(&id) {
            Some(value) => {
                assert_eq!(
                    res.expect("expected value should exist"),
                    DelayedFieldValue::Aggregator(*value)
                );
            }
            None => {
                assert!(matches!(res, Err(MVDelayedFieldsError::NotFound)));
            }
        }
    }

    restarts
}

#[test]
fn delayed_field_combinatorial_no_gas_limit() {
    let seeds = [1u64, 2, 3, 4];
    for seed in seeds {
        let (txns, ids) = gen_transactions(seed, 120, 10, 1..=3, 20);
        assert_parallel_matches_expected(txns, ids, None);
    }
}

#[test]
fn delayed_field_combinatorial_with_gas_limit() {
    let seeds = [10u64, 11, 12];
    for seed in seeds {
        let (txns, ids) = gen_transactions(seed, 200, 12, 1..=4, 20);
        assert_parallel_matches_expected(txns, ids, Some(150));
    }
}

#[test]
fn delayed_field_disjoint_ids_no_restart() {
    let (txns, ids) = gen_transactions(7, 64, 64, 1..=2, 10);
    let restarts = assert_parallel_matches_expected(txns, ids, None);
    assert_eq!(restarts, 0, "disjoint ids should not trigger BlockRestart");
}
