// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod storage_wrapper;
mod vm_wrapper;

use crate::metrics::VMMetrics;
use crate::{
    adapter_common::{preprocess_transaction, PreprocessedTransaction},
    parallel_executor::vm_wrapper::StarcoinVMWrapper,
    starcoin_vm::StarcoinVM,
};
use move_core_types::vm_status::{StatusCode, VMStatus};
use rayon::prelude::*;
use starcoin_parallel_executor::{
    errors::Error,
    executor::ParallelTransactionExecutor,
    task::{Transaction as PTransaction, TransactionOutput as PTransactionOutput},
};
use starcoin_vm_types::{
    state_store::state_key::StateKey,
    state_view::StateView,
    transaction::{Transaction, TransactionOutput, TransactionStatus},
    write_set::{WriteOp, WriteSet},
};

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
            WriteSet::default(),
            vec![],
            0,
            TransactionStatus::Retry,
        ))
    }
}

pub struct ParallelStarcoinVM();

impl ParallelStarcoinVM {
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

        match ParallelTransactionExecutor::<PreprocessedTransaction, StarcoinVMWrapper<S>>::new(
            concurrency_level,
        )
        .execute_transactions_parallel(state_view, signature_verified_block)
        {
            Ok(results) => Ok((
                results
                    .into_iter()
                    .map(StarcoinTransactionOutput::into)
                    .collect(),
                None,
            )),
            Err(err @ Error::InferencerError) | Err(err @ Error::UnestimatedWrite) => {
                // XXX FIXME YSG
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
}
