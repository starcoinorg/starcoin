// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockIdAndNumber;
use crate::startup_info::ChainInfo;
use serde::{Deserialize, Serialize};
use starcoin_uint::U256;

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Debug)]
pub enum SyncState {
    /// Prepare to check sync status
    Prepare,
    /// Node is synchronizing, BlockIdAndNumber is target.
    Synchronizing {
        target: BlockIdAndNumber,
        total_difficulty: U256,
    },
    /// Node is synchronized with peers.
    Synchronized,
}

impl SyncState {
    pub fn is_prepare(&self) -> bool {
        match self {
            SyncState::Prepare => true,
            _ => false,
        }
    }

    pub fn is_syncing(&self) -> bool {
        match self {
            SyncState::Synchronizing { .. } => true,
            _ => false,
        }
    }

    pub fn is_synced(&self) -> bool {
        match self {
            SyncState::Synchronized => true,
            _ => false,
        }
    }
}

pub const NEARLY_SYNCED_BLOCKS: u64 = 24;

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Debug)]
pub struct SyncStatus {
    chain_info: ChainInfo,
    state: SyncState,
}

impl SyncStatus {
    pub fn new(chain_info: ChainInfo) -> Self {
        Self {
            chain_info,
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

    pub fn update_chain_info(&mut self, chain_info: ChainInfo) -> bool {
        if self.chain_info != chain_info {
            self.chain_info = chain_info;
            return true;
        }
        false
    }

    pub fn sync_status(&self) -> &SyncState {
        &self.state
    }

    pub fn chain_info(&self) -> &ChainInfo {
        &self.chain_info
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
                target.number.saturating_sub(self.chain_info.head().number) <= NEARLY_SYNCED_BLOCKS
                    || self.chain_info.total_difficulty() >= *total_difficulty
            }
        }
    }

    pub fn is_synced(&self) -> bool {
        match &self.state {
            SyncState::Prepare => false,
            SyncState::Synchronized => true,
            SyncState::Synchronizing {
                target,
                total_difficulty,
            } => {
                self.chain_info.head().number >= target.number
                    || self.chain_info.total_difficulty() >= *total_difficulty
            }
        }
    }

    pub fn is_prepare(&self) -> bool {
        self.state.is_prepare()
    }
}
