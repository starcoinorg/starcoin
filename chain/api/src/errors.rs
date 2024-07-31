// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Error;
use network_api::ReputationChange;
use starcoin_types::block::{Block, BlockHeader};
use std::fmt::{Display, Formatter};
use thiserror::Error;

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
            return Err(ConnectBlockError::VerifyBlockFailed($verify_field, anyhow::anyhow!($err)).into())
        }
    };
    ($verify_field:expr, $cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err(ConnectBlockError::VerifyBlockFailed($verify_field, anyhow::anyhow!($fmt,  $($arg)*)).into());
        }
    };
}

#[derive(Debug)]
pub enum VerifyBlockField {
    Header,
    Body,
    Uncle,
    Consensus,
    // block field verified base on block executed result.
    State,
}

impl Display for VerifyBlockField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Body => write!(f, "body"),
            Self::Header => write!(f, "header"),
            Self::Uncle => write!(f, "uncle"),
            Self::Consensus => write!(f, "consensus"),
            Self::State => write!(f, "state"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConnectBlockError {
    #[error("Future block: {:?} ", .0.header())]
    FutureBlock(Box<Block>),
    #[error("Block {0:?} 's parent not exist")]
    ParentNotExist(Box<BlockHeader>),
    #[error("Verify block {0} failed: {1:?}")]
    VerifyBlockFailed(VerifyBlockField, Error),
    #[error("Barnard hard fork block: {:?} ", .0.header())]
    BarnardHardFork(Box<Block>),
}

impl ConnectBlockError {
    // future block do not change the reputation
    pub const REP_FUTURE_BLOCK: ReputationChange = ReputationChange::new(0, "FutureBlock");
    pub const REP_PARENT_NOT_EXIST: ReputationChange =
        ReputationChange::new_fatal("ParentNotExist");
    pub const REP_VERIFY_BLOCK_FAILED: ReputationChange =
        ReputationChange::new_fatal("VerifyBlockFailed");
    pub const REP_BARNARD_HARD_FORK: ReputationChange =
        ReputationChange::new_fatal("BarnardHardFork");

    pub fn reason(&self) -> &str {
        match self {
            Self::FutureBlock(_) => "FutureBlock",
            Self::ParentNotExist(_) => "ParentNotExist",
            Self::VerifyBlockFailed(_, _) => "VerifyBlockFailed",
            Self::BarnardHardFork(_) => "BarnardHardFork",
        }
    }

    pub fn reputation(&self) -> ReputationChange {
        match self {
            Self::FutureBlock(_) => Self::REP_FUTURE_BLOCK,
            Self::ParentNotExist(_) => Self::REP_PARENT_NOT_EXIST,
            Self::VerifyBlockFailed(_, _) => Self::REP_VERIFY_BLOCK_FAILED,
            Self::BarnardHardFork(_) => Self::REP_BARNARD_HARD_FORK,
        }
    }
}
