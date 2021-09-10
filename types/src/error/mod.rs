// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_vm_types::vm_status::DiscardedVMStatus;
use std::error::Error;
use thiserror::Error;

pub type ExecutorResult<T> = anyhow::Result<T, BlockExecutorError>;

#[derive(Error, Debug)]
pub enum BlockExecutorError {
    #[error("block transaction execute discard, status:{0:?}, transaction_id: {1}")]
    BlockTransactionDiscard(DiscardedVMStatus, HashValue),
    #[error("block transaction accumulator append error")]
    BlockAccumulatorAppendErr,
    #[error("block accumulator get proof error")]
    BlockAccumulatorGetProofErr,
    #[error("block accumulator proof verify error")]
    BlockAccumulatorVerifyErr(HashValue, u64),
    #[error("block chain state read or write error：{0:?}")]
    BlockChainStateErr(anyhow::Error),
    #[error("block accumulator flush error")]
    BlockAccumulatorFlushErr,
    #[error("block transaction execute error, {0:?}")]
    BlockTransactionExecuteErr(anyhow::Error),
    // service error
    #[error("account error, {0:?}")]
    AccountError(anyhow::Error),
    #[error("other error: {0:?}")]
    OtherError(Box<dyn Error + Send + Sync + 'static>),
}
