// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_vm_types::delayed_values::delayed_field_id::DelayedFieldID;
use rand::{rngs::StdRng, Rng, SeedableRng};
use starcoin_aggregator::bounded_math::SignedU128;
use starcoin_aggregator::delayed_change::{DelayedApplyChange, DelayedChange};
use starcoin_aggregator::delta_change_set::DeltaWithMax;
use starcoin_aggregator::types::{DelayedFieldValue, ReadPosition};
use starcoin_mvhashmap::types::MVDelayedFieldsError;
use starcoin_mvhashmap::versioned_delayed_fields::TVersionedDelayedFieldView;
use starcoin_parallel_executor::executor::ParallelTransactionExecutor;
use starcoin_parallel_executor::task::{
    ExecutionStatus, ExecutorTask, Transaction, TransactionOutput,
};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Clone, Debug)]
struct MixTxn {
    reads: Vec<u64>,
    writes: Vec<u64>,
    gas: u64,
    salt: u64,
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

impl Transaction for MixTxn {
    type Key = u64;
    type Value = u64;
}

#[derive(Clone, Debug)]
struct MixOutput {
    writes: Vec<(u64, u64)>,
    gas: u64,
    changes: Vec<(DelayedFieldID, DelayedChange<DelayedFieldID>)>,
}

impl TransactionOutput for MixOutput {
    type T = MixTxn;

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
            changes: Vec::new(),
        }
    }

    fn delayed_field_change_set(&self) -> Vec<(DelayedFieldID, DelayedChange<DelayedFieldID>)> {
        self.changes.clone()
    }
}

struct MixExecutor;

impl ExecutorTask for MixExecutor {
    type T = MixTxn;
    type Output = MixOutput;
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
        let mut sum = 0u64;
        for key in &txn.reads {
            sum = sum.wrapping_add(view.read(key).map(|v| *v).unwrap_or(0));
        }

        let mut writes = Vec::with_capacity(txn.writes.len());
        for key in &txn.writes {
            let value = sum.wrapping_add(txn.salt).wrapping_add(*key);
            writes.push((*key, value));
        }

        let changes = match &txn.op {
            DelayedOp::Create { id, value } => vec![(
                *id,
                DelayedChange::Create(DelayedFieldValue::Aggregator(*value)),
            )],
            DelayedOp::Delta { id, delta, max } => vec![(
                *id,
                DelayedChange::Apply(DelayedApplyChange::AggregatorDelta {
                    delta: DeltaWithMax::new(SignedU128::Positive(*delta), *max),
                }),
            )],
        };

        ExecutionStatus::Success(MixOutput {
            writes,
            gas: txn.gas,
            changes,
        })
    }
}

fn gen_transactions(
    seed: u64,
    universe_size: usize,
    txn_count: usize,
    id_count: usize,
    reads_per_txn: std::ops::RangeInclusive<usize>,
    writes_per_txn: std::ops::RangeInclusive<usize>,
    gas_range: std::ops::RangeInclusive<u64>,
    max_delta: u128,
) -> (Vec<MixTxn>, BTreeSet<DelayedFieldID>) {
    let mut rng = StdRng::seed_from_u64(seed);
    let keys: Vec<u64> = (0..universe_size as u64).collect();
    let mut txns = Vec::with_capacity(txn_count);
    let mut all_ids = BTreeSet::new();

    for idx in 0..id_count {
        let id = DelayedFieldID::new_with_width(5000 + idx as u32, 8);
        all_ids.insert(id);
    }
    let ids: Vec<_> = all_ids.iter().copied().collect();

    let create_count = txn_count.min(id_count);
    for (idx, id) in ids.iter().take(create_count).copied().enumerate() {
        let gas = rng.random_range(gas_range.clone());
        let value = rng.random_range(1..=1000);
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

        txns.push(MixTxn {
            reads: reads.into_iter().collect(),
            writes: writes.into_iter().collect(),
            gas,
            salt: idx as u64,
            op: DelayedOp::Create { id, value },
        });
    }

    for idx in create_count..txn_count {
        let gas = rng.random_range(gas_range.clone());
        let id = ids[rng.random_range(0..create_count.max(1))];
        let delta = rng.random_range(1..=max_delta);
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

        txns.push(MixTxn {
            reads: reads.into_iter().collect(),
            writes: writes.into_iter().collect(),
            gas,
            salt: idx as u64,
            op: DelayedOp::Delta {
                id,
                delta,
                max: u128::MAX,
            },
        });
    }

    (txns, all_ids)
}

fn expected_after_sequential(
    txns: &[MixTxn],
    gas_limit: Option<u64>,
) -> (HashMap<u64, u64>, HashMap<DelayedFieldID, u128>, usize) {
    let mut state = HashMap::new();
    let mut delayed_state = HashMap::new();
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

        match &txn.op {
            DelayedOp::Create { id, value } => {
                delayed_state.insert(*id, *value);
            }
            DelayedOp::Delta { id, delta, max } => {
                let entry = delayed_state
                    .get_mut(id)
                    .expect("delta should only happen after create");
                let delta = DeltaWithMax::new(SignedU128::Positive(*delta), *max);
                *entry = delta.apply_to(*entry).expect("delta should fit");
            }
        }
    }

    (state, delayed_state, executed)
}

fn assert_parallel_matches_expected(
    txns: Vec<MixTxn>,
    all_ids: BTreeSet<DelayedFieldID>,
    gas_limit: Option<u64>,
) {
    let executor: ParallelTransactionExecutor<MixTxn, MixExecutor> =
        ParallelTransactionExecutor::new(num_cpus::get().max(2), gas_limit)
            .with_delayed_fields(true);

    let (mut outputs, delayed_fields) = executor
        .execute_transactions_parallel_with_delayed_fields((), txns.clone())
        .expect("parallel execution should succeed");
    outputs.sort_by_key(|(idx, _)| *idx);

    let (expected_state, expected_delayed, executed_seq) =
        expected_after_sequential(&txns, gas_limit);
    assert_eq!(outputs.len(), executed_seq);
    for (pos, (idx, _)) in outputs.iter().enumerate() {
        assert_eq!(*idx, pos);
    }

    let mut state = HashMap::new();
    for (_, output) in outputs {
        for (k, v) in output.get_writes() {
            state.insert(k, v);
        }
    }
    assert_eq!(state, expected_state);

    for id in all_ids {
        let res = delayed_fields.read_latest_predicted_value(
            &id,
            executed_seq,
            ReadPosition::AfterCurrentTxn,
        );
        match expected_delayed.get(&id) {
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
}

#[test]
fn delayed_field_mixed_no_gas_limit() {
    for seed in [1u64, 2, 3] {
        let (txns, ids) = gen_transactions(seed, 60, 200, 12, 1..=3, 1..=2, 1..=4, 20);
        assert_parallel_matches_expected(txns, ids, None);
    }
}

#[test]
fn delayed_field_mixed_with_gas_limit() {
    for seed in [10u64, 11, 12] {
        let (txns, ids) = gen_transactions(seed, 60, 240, 12, 1..=3, 1..=3, 1..=5, 20);
        assert_parallel_matches_expected(txns, ids, Some(120));
    }
}
