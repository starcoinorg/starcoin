// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;
mod chain_service;
mod consensus;

pub use chain::{Chain, ChainReader, ChainWriter};
pub use chain_service::{ChainAsyncService, ChainService};
pub use consensus::{Consensus, ConsensusHeader};
use thiserror::Error;

pub type ConnectResult<T> = anyhow::Result<T, ConnectBlockError>;

#[derive(Error, Debug, Clone)]
pub enum ConnectBlockError {
    #[error("block already exist.")]
    DuplicateConn,
    #[error("parent block not exist.")]
    FutureBlock,
    #[error("block verify failed.")]
    VerifyFailed,
    #[error("connect failed, cause : {0:?}")]
    Other(String),
}
