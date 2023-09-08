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
}
