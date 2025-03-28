// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_vm_ext::{MoveResolverExt, SessionId};
use anyhow::Result;
use move_core_types::vm_status::{StatusCode, VMStatus};
use move_vm_runtime::move_vm_adapter::SessionAdapter;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::SignedUserTransactionWithType;
use starcoin_vm_types::{
    block_metadata::BlockMetadata,
    transaction::{
        SignatureCheckedTransaction, SignedUserTransaction, Transaction, TransactionOutput,
        TransactionStatus,
    },
    write_set::WriteSet,
};
use std::collections::BTreeMap;

#[allow(dead_code)]
/// TODO: bring more of the execution logic in starcoin_vm into this file.
pub trait VMAdapter {
    /// Creates a new Session backed by the given storage.
    /// TODO: this doesn't belong in this trait. We should be able to remove
    /// this after redesigning cache ownership model.
    // XXX FIXME YSG, this place we use SessionAdapter, we don't have move_vm_ext::SessionExt
    fn new_session<'r, R: MoveResolverExt>(
        &self,
        remote: &'r R,
        session_id: SessionId,
    ) -> SessionAdapter<'r, '_, R>;

    /// Checks the signature of the given signed transaction and returns
    /// `Ok(SignatureCheckedTransaction)` if the signature is valid.
    fn check_signature(txn: SignedUserTransaction) -> Result<SignatureCheckedTransaction>;

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
    UserTransactionExt(Box<SignedUserTransactionWithType>),
}

#[inline]
pub(crate) fn preprocess_transaction(txn: Transaction) -> PreprocessedTransaction {
    match txn {
        Transaction::BlockMetadata(b) => PreprocessedTransaction::BlockMetadata(b),
        Transaction::UserTransaction(txn) => {
            PreprocessedTransaction::UserTransaction(Box::new(txn))
        }
        Transaction::UserTransactionExt(txn) => {
            PreprocessedTransaction::UserTransactionExt(Box::new(txn))
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
