// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::SessionAdapter;
use crate::move_vm_ext::{MoveResolverExt, SessionId};
use anyhow::Result;
use move_core_types::vm_status::{StatusCode, VMStatus};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::{
    block_metadata::BlockMetadata,
    transaction::{SignedUserTransaction, Transaction, TransactionOutput, TransactionStatus},
    write_set::WriteSet,
};
use std::collections::BTreeMap;

/// TODO: bring more of the execution logic in starcoin_vm into this file.
pub trait VMAdapter {
    /// TODO: maybe remove this after more refactoring of execution logic.
    fn should_restart_execution(output: &TransactionOutput) -> bool;

    /// Execute a single transaction.
    fn execute_single_transaction<S: MoveResolverExt + StateView>(
        &self,
        txn: &PreprocessedTransaction,
        data_cache: &S,
    ) -> Result<(VMStatus, TransactionOutput, Option<String>), VMStatus>;
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
