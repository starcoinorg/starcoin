// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod data_cache;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod natives;
pub mod starcoin_vm;

#[macro_use]
pub mod counters;

use move_core_types::vm_status::{StatusCode, VMStatus};
pub use move_vm_runtime::{move_vm, session};
use std::collections::BTreeMap;
mod access_path_cache;
mod errors;
#[cfg(feature = "force-deploy")]
pub mod force_upgrade_management;
pub mod move_vm_ext;
pub mod parallel_executor;
mod verifier;

use crate::metrics::VMMetrics;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::transaction::{SignedUserTransaction, TransactionStatus};
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
        BTreeMap::new(),
        WriteSet::default(),
        vec![],
        0,
        TransactionStatus::Discard(err),
    )
}
