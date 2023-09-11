// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

use criterion::{measurement::Measurement, BatchSize, Bencher};
use proptest::{
    collection::vec,
    strategy::{Strategy, ValueTree},
    test_runner::TestRunner,
};
use starcoin_crypto::HashValue;
use std::time::{Instant, SystemTime};

use starcoin_language_e2e_tests::account::AccountData;
use starcoin_language_e2e_tests::{
    account_universe::{log_balance_strategy, AUTransactionGen, AccountUniverseGen},
    executor::FakeExecutor,
    gas_costs::TXN_RESERVED,
};

use starcoin_types::{block_metadata::BlockMetadata, transaction::Transaction};

use starcoin_vm_runtime::{block_executor::BlockStarcoinVM, starcoin_vm::StarcoinVM, VMExecutor};
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;

/// Benchmarking support for transactions.
#[derive(Clone, Debug)]
pub struct TransactionBencher<S> {
    num_accounts: usize,
    num_transactions: usize,
    strategy: S,
}

impl<S> TransactionBencher<S>
where
    S: Strategy,
    S::Value: AUTransactionGen,
{
    /// The number of accounts created by default.
    //pub const DEFAULT_NUM_ACCOUNTS: usize = 1000;

    /// The number of transactions created by default.
    // pub const DEFAULT_NUM_TRANSACTIONS: usize = 1000;

    /// Creates a new transaction bencher with default settings.
    pub fn new(strategy: S, num_accounts: usize, num_transactions: usize) -> Self {
        Self {
            num_accounts,
            num_transactions,
            strategy,
        }
    }

    /// Sets a custom number of accounts.
    pub fn num_accounts(&mut self, num_accounts: usize) -> &mut Self {
        self.num_accounts = num_accounts;
        self
    }

    /// Sets a custom number of transactions.
    pub fn num_transactions(&mut self, num_transactions: usize) -> &mut Self {
        self.num_transactions = num_transactions;
        self
    }

    /// Runs the bencher.
    pub fn bench<M: Measurement>(&self, b: &mut Bencher<M>) {
        b.iter_batched(
            || {
                TransactionBenchState::with_size(
                    &self.strategy,
                    self.num_accounts,
                    self.num_transactions,
                )
            },
            |state| state.execute(),
            // The input here is the entire list of signed transactions, so it's pretty large.
            BatchSize::LargeInput,
        )
    }

    /// Runs the bencher.
    pub fn bench_parallel<M: Measurement>(&self, b: &mut Bencher<M>) {
        let start_time = SystemTime::now();
        let num = 8;
        b.iter_batched(
            || {
                ParallelBenchState::with_size(
                    &self.strategy,
                    self.num_accounts,
                    self.num_transactions,
                    num,
                )
            },
            |state| state.execute(),
            // The input here is the entire list of signed transactions, so it's pretty large.
            BatchSize::LargeInput,
        );
        let use_time = SystemTime::now().duration_since(start_time).unwrap();
        println!("cpu num = {}, cost time = {}", num, use_time.as_secs());
    }

    /// Runs the bencher.
    pub fn blockstm_benchmark(
        &self,
        num_accounts: usize,
        num_txn: usize,
        run_par: bool,
        run_seq: bool,
        num_warmups: usize,
        num_runs: usize,
        concurrency_level: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut par_tps = Vec::new();
        let mut seq_tps = Vec::new();

        let total_runs = num_warmups + num_runs;
        for i in 0..total_runs {
            let state = TransactionBenchState::with_size(&self.strategy, num_accounts, num_txn);

            if i < num_warmups {
                println!("WARMUP - ignore results");
                state.execute_blockstm_benchmark(concurrency_level, run_par, run_seq);
            } else {
                println!(
                    "RUN benchmark for: num_threads = {}, \
                        num_account = {}, \
                        block_size = {}",
                    concurrency_level,
                    num_accounts,
                    num_txn,
                );
                let tps = state.execute_blockstm_benchmark(concurrency_level, run_par, run_seq);
                par_tps.push(tps.0);
                seq_tps.push(tps.1);
            }
        }

        (par_tps, seq_tps)
    }

    pub fn manual_sequence(
        &self,
        num_accounts: usize,
        num_txn: usize,
        num_warmups: usize,
        num_runs: usize,
        concurrency_level: usize,
    ) -> Vec<usize> {
        let mut ret = Vec::new();

        let total_runs = num_warmups + num_runs;
        for i in 0..total_runs {
            let state = TransactionBenchState::with_size(&self.strategy, num_accounts, num_txn);

            if i < num_warmups {
                println!("WARMUP - ignore results");
                state.execute();
            } else {
                println!(
                    "RUN bencher for: num_threads = {}, \
                          block_size = {}, \
                          num_account = {}",
                    concurrency_level, num_txn, num_accounts,
                );
                ret.push(state.execute());
            }
        }
        ret
    }

    pub fn manual_parallel(
        &self,
        num_accounts: usize,
        num_txn: usize,
        num_warmups: usize,
        num_runs: usize,
        concurrency_level: usize,
    ) -> Vec<usize> {
        let mut ret = Vec::new();

        let total_runs = num_warmups + num_runs;
        for i in 0..total_runs {
            let state = ParallelBenchState::with_size(
                &self.strategy,
                num_accounts,
                num_txn,
                concurrency_level,
            );

            if i < num_warmups {
                println!("WARMUP - ignore results");
                state.execute();
            } else {
                println!(
                    "RUN bencher for: num_threads = {}, \
                          block_size = {}, \
                          num_account = {}",
                    concurrency_level, num_txn, num_accounts,
                );
                ret.push(state.execute());
            }
        }
        ret
    }
}

struct TransactionBenchState {
    // Use the fake executor for now.
    // TODO: Hook up the real executor in the future. Here's what needs to be done:
    // 1. Provide a way to construct a write set from the genesis write set + initial balances.
    // 2. Provide a trait for an executor with the functionality required for account_universe.
    // 3. Implement the trait for the fake executor.
    // 4. Implement the trait for the real executor, using the genesis write set implemented in 1
    //    and the helpers in the execution_tests crate.
    // 5. Add a type parameter that implements the trait here and switch "executor" to use it.
    // 6. Add an enum to TransactionBencher that lets callers choose between the fake and real
    //    executors.
    executor: FakeExecutor,
    transactions: Vec<Transaction>,
}

impl TransactionBenchState {
    /// Creates a new benchmark state with the given number of accounts and transactions.
    fn with_size<S>(strategy: S, num_accounts: usize, num_transactions: usize) -> Self
    where
        S: Strategy,
        S::Value: AUTransactionGen,
    {
        let mut state = Self::with_universe(
            strategy,
            universe_strategy(num_accounts, num_transactions),
            num_transactions,
        );

        // TODO(BobOng): e2e-test
        // Insert a blockmetadata transaction at the beginning to better simulate the real life traffic.
        // let validator_set =
        //     ValidatorSet::fetch_config(&state.executor.get_state_view().as_move_resolver())
        //         .expect("Unable to retrieve the validator set from storage");
        // let new_block = BlockMetadata::new(
        //     HashValue::zero(),
        //     0,
        //     0,
        //     validator_set.payload().map(|_| false).collect(),
        //     *validator_set.payload().next().unwrap().account_address(),
        //     vec![],
        //     1,
        // );
        let minter_account = AccountData::new(10000, 0);
        let new_block = BlockMetadata::new(
            HashValue::zero(),
            0,
            minter_account.address().clone(),
            Some(AuthenticationKey::ed25519(&minter_account.account().pubkey)),
            0,
            0,
            ChainId::test(),
            0,
        );
        state
            .transactions
            .insert(0, Transaction::BlockMetadata(new_block));

        state
    }

    /// Creates a new benchmark state with the given account universe strategy and number of
    /// transactions.
    fn with_universe<S>(
        strategy: S,
        universe_strategy: impl Strategy<Value = AccountUniverseGen>,
        num_transactions: usize,
    ) -> Self
    where
        S: Strategy,
        S::Value: AUTransactionGen,
    {
        let mut runner = TestRunner::default();
        let universe = universe_strategy
            .new_tree(&mut runner)
            .expect("creating a new value should succeed")
            .current();

        let mut executor = FakeExecutor::from_test_genesis();
        // Run in gas-cost-stability mode for now -- this ensures that new accounts are ignored.
        // XXX We may want to include new accounts in case they have interesting performance
        // characteristics.
        let mut universe = universe.setup_gas_cost_stability(&mut executor);

        let transaction_gens = vec(strategy, num_transactions)
            .new_tree(&mut runner)
            .expect("creating a new value should succeed")
            .current();
        let transactions = transaction_gens
            .into_iter()
            .map(|txn_gen| Transaction::UserTransaction(txn_gen.apply(&mut universe).0))
            .collect();

        Self {
            executor,
            transactions,
        }
    }

    /// Executes this state in a single block.
    fn execute(self) -> usize {
        // The output is ignored here since we're just testing transaction performance, not trying
        // to assert correctness.
        StarcoinVM::set_concurrency_level_once(1);

        let transactions_len = self.transactions.len();

        // this bench execution with TPS
        let timer = Instant::now();
        let useless = StarcoinVM::execute_block(
            self.transactions,
            self.executor.get_state_view(),
            None,
            None,
        )
        .expect("VM should not fail to start");

        drop(useless);

        let exec_t = timer.elapsed();
        transactions_len * 1000 / exec_t.as_millis() as usize
    }

    // /// Executes this state in a single block via parallel execution.
    // fn execute_parallel(self) {
    //     // The output is ignored here since we're just testing transaction performance, not trying
    //     // to assert correctness.
    //     ParallelStarcoinVM::execute_block(
    //         self.transactions,
    //         self.executor.get_state_view(),
    //         num_cpus::get(),
    //         None,
    //         None,
    //     )
    //     .expect("VM should not fail to start");
    // }

    fn execute_blockstm_benchmark(
        self,
        concurrency_level: usize,
        run_par: bool,
        run_seq: bool,
    ) -> (usize, usize) {
        BlockStarcoinVM::execute_block_benchmark(
            self.transactions,
            self.executor.get_state_view(),
            concurrency_level,
            run_par,
            run_seq,
        )
    }
}

/// Returns a strategy for the account universe customized for benchmarks.
fn universe_strategy(
    num_accounts: usize,
    num_transactions: usize,
) -> impl Strategy<Value = AccountUniverseGen> {
    // Multiply by 5 past the number of  to provide
    let max_balance = TXN_RESERVED * num_transactions as u64 * 5;
    let balance_strategy = log_balance_strategy(max_balance);
    AccountUniverseGen::strategy(num_accounts, balance_strategy)
}

struct ParallelBenchState {
    bench_state: TransactionBenchState,
    num_threads: usize,
}

impl ParallelBenchState {
    /// Creates a new benchmark state with the given number of accounts and transactions.
    fn with_size<S>(
        strategy: S,
        num_accounts: usize,
        num_transactions: usize,
        num_threads: usize,
    ) -> Self
    where
        S: Strategy,
        S::Value: AUTransactionGen,
    {
        Self {
            bench_state: TransactionBenchState::with_universe(
                strategy,
                universe_strategy(num_accounts, num_transactions),
                num_transactions,
            ),
            num_threads,
        }
    }

    fn execute(self) -> usize {
        // let txns = self
        //     .bench_state
        //     .transactions
        //     .into_iter()
        //     .map(Transaction::UserTransaction)
        //     .collect();

        let state_view = self.bench_state.executor.get_state_view();
        // measured - microseconds.
        BlockStarcoinVM::execute_block_tps(
            self.bench_state.transactions.clone(),
            state_view,
            self.num_threads,
        )
    }
}
