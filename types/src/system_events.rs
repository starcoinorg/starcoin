// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{Block, BlockHeader, BlockHeaderExtra, ExecutedBlock};
use crate::sync_status::SyncStatus;
use crate::U256;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct NewHeadBlock(pub Arc<ExecutedBlock>);

/// may be uncle block
#[derive(Clone, Debug)]
pub struct NewBranch(pub Arc<[BlockHeader]>);

#[derive(Clone, Debug)]
pub struct MinedBlock(pub Arc<Block>);

///Fire this event on System start and all service is init.
#[derive(Clone, Debug)]
pub struct SystemStarted;

#[derive(Clone, Debug)]
pub struct SyncStatusChangeEvent(pub SyncStatus);

///Fire this event for generate a new block
#[derive(Clone, Debug)]
pub struct GenerateBlockEvent {
    /// Force break current minting, and Generate new block.
    pub force: bool,
}

impl GenerateBlockEvent {
    pub fn new(force: bool) -> Self {
        Self { force }
    }
}

#[derive(Clone, Debug)]
pub struct MintBlockEvent {
    pub strategy: ConsensusStrategy,
    pub minting_blob: Vec<u8>,
    pub difficulty: U256,
    pub block_number: u64,
}

impl MintBlockEvent {
    pub fn new(
        strategy: ConsensusStrategy,
        minting_blob: Vec<u8>,
        difficulty: U256,
        block_number: u64,
    ) -> Self {
        Self {
            strategy,
            minting_blob,
            difficulty,
            block_number,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubmitSealEvent {
    pub nonce: u32,
    pub extra: BlockHeaderExtra,
    pub minting_blob: Vec<u8>,
}

impl SubmitSealEvent {
    pub fn new(minting_blob: Vec<u8>, nonce: u32, extra: BlockHeaderExtra) -> Self {
        Self {
            minting_blob,
            nonce,
            extra,
        }
    }
}
