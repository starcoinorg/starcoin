// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{Block, BlockDetail, BlockHeader};
use crate::sync_status::SyncStatus;
use crate::U256;
use actix::prelude::*;
use anyhow::Result;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::sync::Arc;

//TODO this type should at another crate and avoid starcoin-types dependency actix ?.
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct NewHeadBlock(pub Arc<BlockDetail>);

/// may be uncle block
#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct NewBranch(pub Arc<Vec<BlockHeader>>);

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct MinedBlock(pub Arc<Block>);

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

#[derive(Clone, Debug)]
pub struct SyncStatusChangeEvent(pub SyncStatus);

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
    pub strategy: ConsensusStrategy,
    pub minting_blob: Vec<u8>,
    pub difficulty: U256,
}

impl MintBlockEvent {
    pub fn new(strategy: ConsensusStrategy, minting_blob: Vec<u8>, difficulty: U256) -> Self {
        Self {
            strategy,
            minting_blob,
            difficulty,
        }
    }
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct SubmitSealEvent {
    pub nonce: u32,
    pub minting_blob: Vec<u8>,
}

impl SubmitSealEvent {
    pub fn new(minting_blob: Vec<u8>, nonce: u32) -> Self {
        Self {
            minting_blob,
            nonce,
        }
    }
}
