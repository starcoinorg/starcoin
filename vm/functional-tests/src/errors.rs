// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use anyhow::{anyhow, bail, format_err, Error, Result};
use starcoin_vm_types::transaction::TransactionOutput;
use thiserror::Error;
use vm_status_translator::VmStatusExplainView;

/// Defines all errors in this crate.
#[derive(Clone, Debug, Error)]
#[allow(clippy::upper_case_acronyms)]
pub enum ErrorKind {
    #[error(
    "an error occurred when executing the transaction, vm status {:?}, txn status {:?}",
    .0,
    .1.status(),
    )]
    VMExecutionFailure(VmStatusExplainView, TransactionOutput),
    #[error("the transaction was discarded: {0:?}")]
    DiscardedTransaction(TransactionOutput),
    #[error("the checker has failed to match the directives against the output")]
    CheckerFailure,
    #[error("VerificationError({0:?})")]
    VerificationError(VmStatusExplainView),
    #[error("other error: {0}")]
    #[allow(dead_code)]
    Other(String),
}
