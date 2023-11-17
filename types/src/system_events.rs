// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{Block, BlockHeaderExtra, ExecutedBlock};
use crate::sync_status::SyncStatus;
use crate::U256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct NewHeadBlock(pub Arc<ExecutedBlock>);

/// may be uncle block
#[derive(Clone, Debug)]
pub struct NewBranch(pub Arc<ExecutedBlock>);

#[derive(Clone, Debug)]
pub struct MinedBlock(pub Arc<Block>);

///Fire this event on System start and all service is init.
#[derive(Clone, Debug)]
pub struct SystemStarted;

#[derive(Clone, Debug)]
pub struct SystemShutdown;

#[derive(Clone, Debug)]
pub struct SyncStatusChangeEvent(pub SyncStatus);

///Fire this event for generate a new block
#[derive(Clone, Debug)]
pub struct GenerateBlockEvent {
    /// Force break current minting task, and Generate new block.
    pub break_current_task: bool,
    /// Skip the empty block check, see MinerConfig::disable_mint_empty_block
    pub skip_empty_block_check: bool,
}

impl Default for GenerateBlockEvent {
    fn default() -> Self {
        Self::new(false, false)
    }
}

impl GenerateBlockEvent {
    pub fn new(break_current_task: bool, skip_empty_block_check: bool) -> Self {
        Self {
            break_current_task,
            skip_empty_block_check,
        }
    }
    pub fn new_break(break_current_task: bool) -> Self {
        Self {
            break_current_task,
            skip_empty_block_check: false,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MintBlockEvent {
    pub parent_hash: HashValue,
    pub strategy: ConsensusStrategy,
    #[serde(with = "hex")]
    #[schemars(with = "String")]
    pub minting_blob: Vec<u8>,
    #[schemars(with = "String")]
    pub difficulty: U256,
    pub block_number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<MintEventExtra>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MintEventExtra {
    pub worker_id: String,
    pub job_id: String,
    pub extra: BlockHeaderExtra,
}

impl MintBlockEvent {
    pub fn new(
        parent_hash: HashValue,
        strategy: ConsensusStrategy,
        minting_blob: Vec<u8>,
        difficulty: U256,
        block_number: u64,
        extra: Option<MintEventExtra>,
    ) -> Self {
        Self {
            parent_hash,
            strategy,
            minting_blob,
            difficulty,
            block_number,
            extra,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SealEvent {
    pub minting_blob: Vec<u8>,
    pub nonce: u32,
    pub extra: Option<MintEventExtra>,
    pub hash_result: String,
}
