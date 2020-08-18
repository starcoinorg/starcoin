// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
use starcoin_types::block::{Block, BlockHeader};
use std::fmt::{Display, Formatter};
use thiserror::Error;

mod chain;
mod chain_service;

pub use chain::{Chain, ChainReader, ChainWriter, ExcludedTxns};
pub use chain_service::{ChainAsyncService, ChainService};

#[macro_export]
macro_rules! verify_block {
    ($verify_field:expr, $cond:expr) => {
        if !$cond {
            return Err(ConnectBlockError::VerifyBlockFailed($verify_field, anyhow::anyhow!("check condition {} fail")).into())
        }
    };
    ($verify_field:expr, $cond:expr, $msg:literal) => {
        if !$cond {
            return Err(ConnectBlockError::VerifyBlockFailed($verify_field, anyhow::anyhow!($msg)).into())
        }
    };
    ($verify_field:expr, $cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return Err(ConnectBlockError::VerifyBlockFailed($verify_field, anyhow::anyhow!($msg)).into())
        }
    };
    ($verify_field:expr, $cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err(ConnectBlockError::VerifyBlockFailed($verify_field, anyhow::anyhow!($fmt,  $($arg)*)).into());
        }
    };
}

#[derive(Debug)]
pub enum BlockVerifyField {
    Header,
    Body,
    Uncle,
    Consensus,
}

impl Display for BlockVerifyField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockVerifyField::Body => write!(f, "body"),
            BlockVerifyField::Header => write!(f, "header"),
            BlockVerifyField::Uncle => write!(f, "uncle"),
            BlockVerifyField::Consensus => write!(f, "consensus"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConnectBlockError {
    #[error("DuplicateConn block: {:?} ", .0.header())]
    DuplicateConn(Box<Block>),
    #[error("Future block: {:?} ", .0.header())]
    FutureBlock(Box<Block>),
    #[error("Block {0:?} 's parent not exist")]
    ParentNotExist(Box<BlockHeader>),
    #[error("Verify block {0} failed: {1:?}")]
    VerifyBlockFailed(BlockVerifyField, Error),
}
