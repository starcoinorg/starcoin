// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod storage_wrapper;
mod vm_wrapper;

use crate::{
    adapter_common::{preprocess_transaction, PreprocessedTransaction},
    block_executor::vm_wrapper::StarcoinVMWrapper,
    metrics::VMMetrics,
    starcoin_vm::StarcoinVM,
};
use move_core_types::vm_status::{StatusCode, VMStatus};
use rayon::prelude::*;
use starcoin_block_executor::executor::RAYON_EXEC_POOL;
use starcoin_block_executor::{
    errors::Error,
    executor::BlockExecutor,
    task::{Transaction as PTransaction, TransactionOutput as PTransactionOutput},
};
use starcoin_vm_types::{
    state_store::state_key::StateKey,
    state_view::StateView,
    transaction::{Transaction, TransactionOutput, TransactionStatus},
    write_set::{WriteOp, WriteSet},
};
use std::collections::BTreeMap;
use std::time::Instant;

impl PTransaction for PreprocessedTransaction {
    type Key = StateKey;
    type Value = WriteOp;
}

// Wrapper to avoid orphan rule
pub(crate) struct StarcoinTransactionOutput(TransactionOutput);

impl StarcoinTransactionOutput {
    pub fn new(output: TransactionOutput) -> Self {
        Self(output)
    }
    pub fn into(self) -> TransactionOutput {
        self.0
    }
}

impl PTransactionOutput for StarcoinTransactionOutput {
    type T = PreprocessedTransaction;

    fn get_writes(&self) -> Vec<(StateKey, WriteOp)> {
        self.0.write_set().iter().cloned().collect()
    }

    /// Execution output for transactions that comes after SkipRest signal.
    fn skip_output() -> Self {
        Self(TransactionOutput::new(
            BTreeMap::new(),
            WriteSet::default(),
            vec![],
            0,
            TransactionStatus::Retry,
        ))
    }
}

pub struct BlockStarcoinVM();

impl BlockStarcoinVM {
    pub fn execute_block<S: StateView>(
        transactions: Vec<Transaction>,
        state_view: &S,
        concurrency_level: usize,
        block_gas_limit: Option<u64>,
        metrics: Option<VMMetrics>,
    ) -> Result<(Vec<TransactionOutput>, Option<Error<VMStatus>>), VMStatus> {
        let signature_verified_block: Vec<PreprocessedTransaction> = transactions
            .par_iter()
            .map(|txn| preprocess_transaction(txn.clone()))
            .collect();

        // XXX FIXME YSG block_gas_limit
        match BlockExecutor::<PreprocessedTransaction, StarcoinVMWrapper<S>>::new(concurrency_level)
            .execute_transactions_parallel(state_view, signature_verified_block)
        {
            Ok(results) => Ok((
                results
                    .into_iter()
                    .map(StarcoinTransactionOutput::into)
                    .collect(),
                None,
            )),
            // XXX FIXME YSG
            Err(err @ Error::BlockRestart) | Err(err @ Error::ModulePathReadWrite) => {
                let output = StarcoinVM::execute_block_and_keep_vm_status(
                    transactions,
                    state_view,
                    block_gas_limit,
                    metrics,
                )?;
                Ok((
                    output
                        .into_iter()
                        .map(|(_vm_status, txn_output)| txn_output)
                        .collect(),
                    Some(err),
                ))
            }
            Err(Error::InvariantViolation) => Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            )),
            Err(Error::UserError(err)) => Err(err),
        }
    }

    pub fn execute_block_tps<S: StateView>(
        transactions: Vec<Transaction>,
        state_view: &S,
        parallel_level: usize,
    ) -> usize {
        // Verify the signatures of all the transactions in parallel.
        // This is time consuming so don't wait and do the checking
        // sequentially while executing the transactions.

        // let mut timer = Instant::now();
        let signature_verified_block: Vec<PreprocessedTransaction> = transactions
            .par_iter()
            .map(|txn| preprocess_transaction(txn.clone()))
            .collect();
        // println!("CLONE & Prologue {:?}", timer.elapsed());

        let executor =
            BlockExecutor::<PreprocessedTransaction, StarcoinVMWrapper<S>>::new(parallel_level);

        let timer = Instant::now();
        let useless = executor.execute_transactions_parallel(state_view, signature_verified_block);
        let exec_t = timer.elapsed();

        drop(useless);

        transactions.len() * 1000 / exec_t.as_millis() as usize
    }

    pub fn execute_block_benchmark<S: StateView + Sync>(
        transactions: Vec<Transaction>,
        state_view: &S,
        concurrency_level: usize,
        run_par: bool,
        run_seq: bool,
    ) -> (usize, usize) {
        let (par_tps, par_ret) = if run_par {
            BlockStarcoinVM::execute_block_benchmark_parallel(
                transactions.clone(),
                state_view,
                concurrency_level,
            )
        } else {
            (0, None)
        };
        let (seq_tps, seq_ret) = if run_seq {
            BlockStarcoinVM::execute_block_benchmark_sequential(transactions, state_view)
        } else {
            (0, None)
        };

        if let (Some(par), Some(seq)) = (par_ret.as_ref(), seq_ret.as_ref()) {
            assert_eq!(par, seq);
        }

        drop(par_ret);
        drop(seq_ret);

        (par_tps, seq_tps)
    }

    fn execute_block_benchmark_parallel<S: StateView + Sync>(
        transactions: Vec<Transaction>,
        state_view: &S,
        concurrency_level: usize,
    ) -> (
        usize,
        Option<Result<Vec<TransactionOutput>, Error<VMStatus>>>,
    ) {
        // Verify the signatures of all the transactions in parallel.
        // This is time consuming so don't wait and do the checking
        // sequentially while executing the transactions.
        let signature_verified_block: Vec<PreprocessedTransaction> =
            RAYON_EXEC_POOL.install(|| {
                transactions
                    .clone()
                    .into_par_iter()
                    .with_min_len(25)
                    .map(preprocess_transaction)
                    .collect()
            });
        let block_size = signature_verified_block.len();

        let executor =
            BlockExecutor::<PreprocessedTransaction, StarcoinVMWrapper<S>>::new(concurrency_level);
        println!("Parallel execution starts...");
        let timer = Instant::now();
        let ret = executor.execute_transactions_parallel(state_view, signature_verified_block);
        let exec_t = timer.elapsed();
        println!(
            "Parallel execution finishes, TPS = {}",
            block_size * 1000 / exec_t.as_millis() as usize
        );
        let par_ret = ret.map(|results| results.into_iter().map(|output| output.into()).collect());
        (
            block_size * 1000 / exec_t.as_millis() as usize,
            Some(par_ret),
        )
    }

    fn execute_block_benchmark_sequential<S: StateView + Sync>(
        transactions: Vec<Transaction>,
        state_view: &S,
    ) -> (
        usize,
        Option<Result<Vec<TransactionOutput>, Error<VMStatus>>>,
    ) {
        // Verify the signatures of all the transactions in parallel.
        // This is time consuming so don't wait and do the checking
        // sequentially while executing the transactions.

        let block_size = transactions.len();

        // sequentially execute the block and check if the results match
        println!("Sequential execution starts...");
        let seq_timer = Instant::now();
        let mut vm = StarcoinVM::new(None);
        let ret = vm.execute_block_transactions(state_view, transactions, None);
        let seq_exec_t = seq_timer.elapsed();
        println!(
            "Sequential execution finishes, TPS = {}",
            block_size * 1000 / seq_exec_t.as_millis() as usize
        );

        (block_size * 1000 / seq_exec_t.as_millis() as usize, None)
    }
}
