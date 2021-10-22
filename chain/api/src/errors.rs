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
            VerifyBlockField::Body => write!(f, "body"),
            VerifyBlockField::Header => write!(f, "header"),
            VerifyBlockField::Uncle => write!(f, "uncle"),
            VerifyBlockField::Consensus => write!(f, "consensus"),
            VerifyBlockField::State => write!(f, "state"),
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
}

impl ConnectBlockError {
    // future block do not change the reputation
    pub const REP_FUTURE_BLOCK: ReputationChange = ReputationChange::new(0, "FutureBlock");
    pub const REP_PARENT_NOT_EXIST: ReputationChange =
        ReputationChange::new_fatal("ParentNotExist");
    pub const REP_VERIFY_BLOCK_FAILED: ReputationChange =
        ReputationChange::new_fatal("VerifyBlockFailed");

    pub fn reason(&self) -> &str {
        match self {
            ConnectBlockError::FutureBlock(_) => "FutureBlock",
            ConnectBlockError::ParentNotExist(_) => "ParentNotExist",
            ConnectBlockError::VerifyBlockFailed(_, _) => "VerifyBlockFailed",
        }
    }

    pub fn reputation(&self) -> ReputationChange {
        match self {
            ConnectBlockError::FutureBlock(_) => ConnectBlockError::REP_FUTURE_BLOCK,
            ConnectBlockError::ParentNotExist(_) => ConnectBlockError::REP_PARENT_NOT_EXIST,
            ConnectBlockError::VerifyBlockFailed(_, _) => {
                ConnectBlockError::REP_VERIFY_BLOCK_FAILED
            }
        }
    }
}
