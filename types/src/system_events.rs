// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{Block, BlockDetail, BlockHeader};
use crate::U256;
use actix::prelude::*;
use anyhow::Result;
use starcoin_crypto::HashValue;
use std::sync::Arc;

//TODO this type should at another crate and avoid starcoin-types dependency actix ?.
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct NewHeadBlock(pub Arc<BlockDetail>);

/// may be uncle block
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct NewBranch(pub Arc<BlockHeader>);

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct MinedBlock(pub Arc<Block>);

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SyncBegin;

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SyncDone;

/// Try to stop a actor
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct ActorStop;

/// Try to stop system.
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SystemStop;

///Fire this event on System start and all service is init.
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct SystemStarted;

///Fire this event for generate a new block
#[derive(Clone, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateBlockEvent {
    /// Force break current minting, and Generate new block.
    pub force: bool,
}

impl GenerateBlockEvent {
    pub fn new(force: bool) -> Self {
        Self { force }
    }
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct MintBlockEvent {
    pub header_hash: HashValue,
    pub difficulty: U256,
}

impl MintBlockEvent {
    pub fn new(header_hash: HashValue, difficulty: U256) -> Self {
        Self {
            header_hash,
            difficulty,
        }
    }
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct SubmitSealEvent {
    pub nonce: u64,
    pub header_hash: HashValue,
}
