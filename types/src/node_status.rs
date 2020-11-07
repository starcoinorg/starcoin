// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockIdAndNumber;
use crate::startup_info::ChainInfo;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Debug)]
pub enum SyncStatus {
    /// Prepare to check sync status
    Prepare,
    /// Node is synchronizing, BlockIdAndNumber is target.
    Synchronizing(BlockIdAndNumber),
    /// Node is synchronized with peers.
    Synchronized,
}

impl SyncStatus {
    pub fn is_prepare(&self) -> bool {
        match self {
            SyncStatus::Prepare => true,
            _ => false,
        }
    }

    pub fn is_syncing(&self) -> bool {
        match self {
            SyncStatus::Synchronizing(_) => true,
            _ => false,
        }
    }

    pub fn is_synced(&self) -> bool {
        match self {
            SyncStatus::Synchronized => true,
            _ => false,
        }
    }
}

pub const NEARLY_SYNCED_BLOCKS: u64 = 6;

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Debug)]
pub struct NodeStatus {
    chain_info: ChainInfo,
    sync_status: SyncStatus,
}

impl NodeStatus {
    pub fn new(chain_info: ChainInfo) -> Self {
        Self {
            chain_info,
            sync_status: SyncStatus::Prepare,
        }
    }

    pub fn update_sync_status(&mut self, sync_status: SyncStatus) -> bool {
        if self.sync_status != sync_status {
            self.sync_status = sync_status;
            return true;
        }
        false
    }

    pub fn update_chain_info(&mut self, chain_info: ChainInfo) -> bool {
        if self.chain_info != chain_info {
            self.chain_info = chain_info;
            return true;
        }
        false
    }

    pub fn sync_status(&self) -> &SyncStatus {
        &self.sync_status
    }

    pub fn chain_info(&self) -> &ChainInfo {
        &self.chain_info
    }

    pub fn is_syncing(&self) -> bool {
        self.sync_status.is_syncing()
    }

    pub fn is_nearly_synced(&self) -> bool {
        match &self.sync_status {
            SyncStatus::Prepare => false,
            SyncStatus::Synchronized => true,
            SyncStatus::Synchronizing(target) => {
                target.number.saturating_sub(self.chain_info.head().number) <= NEARLY_SYNCED_BLOCKS
            }
        }
    }

    pub fn is_synced(&self) -> bool {
        match &self.sync_status {
            SyncStatus::Prepare => false,
            SyncStatus::Synchronized => true,
            SyncStatus::Synchronizing(target) => self.chain_info.head().number >= target.number,
        }
    }

    pub fn is_prepare(&self) -> bool {
        self.sync_status.is_prepare()
    }
}
