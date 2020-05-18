// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_error::VMStatus;
use starcoin_crypto::HashValue;
use std::error::Error;
use thiserror::Error;

pub type ExecutorResult<T> = anyhow::Result<T, BlockExecutorError>;

#[derive(Error, Debug)]
pub enum BlockExecutorError {
    #[error("block transaction execute discard, vmstatus:{0}, transaction_id: {1}")]
    BlockTransactionDiscard(VMStatus, HashValue),
    #[error("block transaction accumulator append error")]
    BlockAccumulatorAppendErr,
    #[error("block accumulator get proof error")]
    BlockAccumulatorGetProofErr,
    #[error("block accumulator proof verify error")]
    BlockAccumulatorVerifyErr(HashValue, u64),
    #[error("block chain state commit error")]
    BlockChainStateCommitErr,
    #[error("block accumulator flush error")]
    BlockAccumulatorFlushErr,
    #[error("block chain state flush error")]
    BlockChainStateFlushErr,
    #[error("block transaction execute error, {0:?}")]
    BlockTransactionExecuteErr(anyhow::Error),
    // service error
    #[error("account error, {0:?}")]
    AccountError(anyhow::Error),
    #[error("other error: {0:?}")]
    OtherError(Box<dyn Error + Send + Sync + 'static>),
}
