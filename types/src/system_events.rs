// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{Block, BlockDetail};
use actix::prelude::*;

//TODO this type should at another crate and avoid starcoin-types dependency actix ?.
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub enum SystemEvents {
    /// Find new head block.
    NewHeadBlock(BlockDetail),
    /// Mint new Block.
    MinedBlock(Block),
    /// Begin to sync
    SyncBegin(),
    /// Sync done
    SyncDone(),
}

impl SystemEvents {
    pub fn is_sync_done(&self) -> bool {
        match self {
            SystemEvents::SyncDone() => true,
            _ => false,
        }
    }
}
