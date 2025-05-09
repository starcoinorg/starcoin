// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod data_cache;
pub mod natives;
pub mod starcoin_vm;

#[macro_use]
pub mod counters;

use move_core_types::vm_status::{StatusCode, VMStatus};
pub use move_vm_runtime::{move_vm, session};
use starcoin_gas_schedule::{
    InitialGasSchedule, StarcoinGasParameters, ToOnChainGasSchedule, LATEST_GAS_FEATURE_VERSION,
};

mod access_path_cache;
mod errors;
pub mod move_vm_ext;
pub mod parallel_executor;
mod verifier;

use starcoin_metrics::metrics::VMMetrics;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::on_chain_config::GasSchedule;
use starcoin_vm_types::transaction::{
    SignedUserTransaction, TransactionAuxiliaryData, TransactionStatus,
};
use starcoin_vm_types::write_set::WriteSet;
use starcoin_vm_types::{
    state_store::StateView,
    transaction::{Transaction, TransactionOutput},
};

/// This trait describes the VM's execution interface.
pub trait VMExecutor: Send + Sync {
    // NOTE: At the moment there are no persistent caches that live past the end of a block (that's
    // why execute_block doesn't take &self.)
    // There are some cache invalidation issues around transactions publishing code that need to be
    // sorted out before that's possible.

    /// Executes a block of transactions and returns output for each one of them.
    fn execute_block(
        transactions: Vec<Transaction>,
        state_view: &(impl StateView + Sync),
        block_gas_limit: Option<u64>,
        metrics: Option<VMMetrics>,
    ) -> Result<Vec<TransactionOutput>, VMStatus>;
}

#[derive(Debug)]
pub enum PreprocessedTransaction {
    UserTransaction(Box<SignedUserTransaction>),
    BlockMetadata(BlockMetadata),
}

#[inline]
pub fn preprocess_transaction(txn: Transaction) -> PreprocessedTransaction {
    match txn {
        Transaction::BlockMetadata(b) => PreprocessedTransaction::BlockMetadata(b),
        Transaction::UserTransaction(txn) => {
            PreprocessedTransaction::UserTransaction(Box::new(txn))
        }
    }
}

pub(crate) fn discard_error_vm_status(err: VMStatus) -> (VMStatus, TransactionOutput) {
    let vm_status = err.clone();
    let error_code = match err.keep_or_discard() {
        Ok(_) => {
            debug_assert!(false, "discarding non-discardable error: {:?}", vm_status);
            vm_status.status_code()
        }
        Err(code) => code,
    };
    (vm_status, discard_error_output(error_code))
}

pub(crate) fn discard_error_output(err: StatusCode) -> TransactionOutput {
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        WriteSet::default(),
        vec![],
        0,
        TransactionStatus::Discard(err),
        TransactionAuxiliaryData::None,
    )
}

pub(crate) fn default_gas_schedule() -> GasSchedule {
    GasSchedule {
        feature_version: LATEST_GAS_FEATURE_VERSION,
        entries: StarcoinGasParameters::initial()
            .to_on_chain_gas_schedule(LATEST_GAS_FEATURE_VERSION),
    }
}
