// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockIdAndNumber;
use crate::startup_info::ChainStatus;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_uint::U256;
#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub enum SyncState {
    /// Prepare to check sync status
    Prepare,
    /// Node is synchronizing, BlockIdAndNumber is target.
    Synchronizing {
        target: BlockIdAndNumber,
        #[schemars(with = "String")]
        total_difficulty: U256,
    },
    /// Node is synchronized with peers.
    Synchronized,
}

impl SyncState {
    pub fn is_prepare(&self) -> bool {
        matches!(self, SyncState::Prepare)
    }

    pub fn is_syncing(&self) -> bool {
        matches!(self, SyncState::Synchronizing { .. })
    }

    pub fn is_synced(&self) -> bool {
        matches!(self, SyncState::Synchronized)
    }
}

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct SyncStatus {
    chain_status: ChainStatus,
    state: SyncState,
}

pub const NEARLY_SYNCED_BLOCKS: u64 = 24;

impl SyncStatus {
    pub fn new(chain_status: ChainStatus) -> Self {
        Self {
            chain_status,
            state: SyncState::Prepare,
        }
    }

    pub fn sync_begin(&mut self, target: BlockIdAndNumber, total_difficulty: U256) {
        self.state = SyncState::Synchronizing {
            target,
            total_difficulty,
        };
    }

    pub fn sync_done(&mut self) {
        self.state = SyncState::Synchronized;
    }

    pub fn update_chain_status(&mut self, chain_status: ChainStatus) -> bool {
        if self.chain_status != chain_status {
            self.chain_status = chain_status;
            return true;
        }
        false
    }

    pub fn sync_status(&self) -> &SyncState {
        &self.state
    }

    pub fn chain_status(&self) -> &ChainStatus {
        &self.chain_status
    }

    pub fn is_syncing(&self) -> bool {
        self.state.is_syncing()
    }

    pub fn is_nearly_synced(&self) -> bool {
        match &self.state {
            SyncState::Prepare => false,
            SyncState::Synchronized => true,
            SyncState::Synchronizing {
                target,
                total_difficulty,
            } => {
                if target.number() < self.chain_status.head().number() {
                    false
                } else {
                    target
                        .number
                        .saturating_sub(self.chain_status.head().number())
                        <= NEARLY_SYNCED_BLOCKS
                        || self.chain_status.total_difficulty() >= *total_difficulty
                }
            }
        }
    }

    pub fn is_synced(&self) -> bool {
        self.state.is_synced()
    }

    pub fn is_prepare(&self) -> bool {
        self.state.is_prepare()
    }
}
