// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor::ParallelTransactionExecutor,
    proptest_types::types::{
        ExpectedOutput, Task, Transaction, TransactionGen, TransactionGenParams,
    },
};
use criterion::{BatchSize, Bencher as CBencher};
use num_cpus;
use proptest::{
    arbitrary::Arbitrary,
    collection::vec,
    prelude::*,
    strategy::{Strategy, ValueTree},
    test_runner::TestRunner,
};

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub struct Bencher<K, V> {
    transaction_size: usize,
    transaction_gen_param: TransactionGenParams,
    universe_size: usize,
    phantom_key: PhantomData<K>,
    phantom_value: PhantomData<V>,
}

pub(crate) struct BencherState<K, V> {
    transactions: Vec<Transaction<K, V>>,
    expected_output: ExpectedOutput<V>,
}

impl<K, V> Bencher<K, V>
where
    K: Hash + Clone + Debug + Eq + Send + Sync + PartialOrd + Ord + Arbitrary + 'static,
    V: Clone + Eq + Send + Sync + Arbitrary + 'static,
{
    pub fn new(transaction_size: usize, universe_size: usize) -> Self {
        Self {
            transaction_size,
            transaction_gen_param: TransactionGenParams::default(),
            universe_size,
            phantom_key: PhantomData,
            phantom_value: PhantomData,
        }
    }

    pub fn bench(&self, key_strategy: &impl Strategy<Value = K>, bencher: &mut CBencher) {
        bencher.iter_batched(
            || {
                BencherState::<K, V>::with_universe(
                    vec(key_strategy, self.universe_size),
                    self.transaction_size,
                    self.transaction_gen_param,
                )
            },
            |state| state.run(),
            // The input here is the entire list of signed transactions, so it's pretty large.
            BatchSize::LargeInput,
        )
    }
}

impl<K, V> BencherState<K, V>
where
    K: Hash + Clone + Debug + Eq + Send + Sync + PartialOrd + Ord + 'static,
    V: Clone + Eq + Send + Sync + Arbitrary + 'static,
{
    /// Creates a new benchmark state with the given account universe strategy and number of
    /// transactions.
    pub(crate) fn with_universe(
        universe_strategy: impl Strategy<Value = Vec<K>>,
        num_transactions: usize,
        transaction_params: TransactionGenParams,
    ) -> Self {
        let mut runner = TestRunner::default();
        let key_universe = universe_strategy
            .new_tree(&mut runner)
            .expect("creating a new value should succeed")
            .current();

        let transaction_gens = vec(
            any_with::<TransactionGen<V>>(transaction_params),
            num_transactions,
        )
        .new_tree(&mut runner)
        .expect("creating a new value should succeed")
        .current();

        let transactions: Vec<_> = transaction_gens
            .into_iter()
            .map(|txn_gen| txn_gen.materialize(&key_universe))
            .collect();

        let expected_output = ExpectedOutput::generate_baseline(&transactions);

        Self {
            transactions,
            expected_output,
        }
    }

    pub(crate) fn run(self) {
        let output =
            ParallelTransactionExecutor::<Transaction<K, V>, Task<K, V>>::new(num_cpus::get())
                .execute_transactions_parallel((), self.transactions.clone());

        assert!(self.expected_output.check_output(&output));
    }
}
