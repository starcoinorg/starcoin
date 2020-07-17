// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

use crate::{access_path::AccessPath, account_config::constants::CORE_CODE_ADDRESS};
use move_core_types::{language_storage::StructTag, move_resource::MoveResource};

const CONSENSUS_MODULE_NAME: &str = "Consensus";

/// Defines the version of Starcoin software.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Consensus {
    pub uncle_rate_target: u64,
    pub epoch_time_target: u64,
    pub reward_half_epoch: u64,
    pub block_difficulty_window: u64,
    pub reward_per_uncle_percent: u64,
    pub min_time_target: u64,
    pub max_uncles_per_block: u64,
}

impl OnChainConfig for Consensus {
    const IDENTIFIER: &'static str = CONSENSUS_MODULE_NAME;
}

/// The Epoch resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct EpochResource {
    epoch_number: u64,
    epoch_start_time: u64,
    start_number: u64,
    end_number: u64,
    block_time_target: u64,
    reward_per_epoch: u64,
    reward_per_block: u64,
}

impl EpochResource {
    pub fn new(
        epoch_number: u64,
        epoch_start_time: u64,
        start_number: u64,
        end_number: u64,
        block_time_target: u64,
        reward_per_epoch: u64,
        reward_per_block: u64,
    ) -> Self {
        Self {
            epoch_number,
            epoch_start_time,
            start_number,
            end_number,
            block_time_target,
            reward_per_epoch,
            reward_per_block,
        }
    }

    pub fn start_number(&self) -> u64 {
        self.start_number
    }

    pub fn block_time_target(&self) -> u64 {
        self.block_time_target
    }

    // TODO/XXX: remove this once the MoveResource trait allows type arguments to `struct_tag`.
    pub fn struct_tag_for_epoch() -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: EpochResource::struct_identifier(),
            module: EpochResource::module_identifier(),
            type_params: vec![],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn access_path_for() -> Vec<u8> {
        AccessPath::resource_access_vec(&EpochResource::struct_tag_for_epoch())
    }
}

impl MoveResource for EpochResource {
    const MODULE_NAME: &'static str = CONSENSUS_MODULE_NAME;
    const STRUCT_NAME: &'static str = "Epoch";
}

/// The Epoch resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct EpochDataResource {
    epoch_number: u64,
    epoch_start_time: u64,
    start_number: u64,
    end_number: u64,
    block_time_target: u64,
    reward_per_epoch: u64,
    reward_per_block: u64,
}

impl MoveResource for EpochDataResource {
    const MODULE_NAME: &'static str = CONSENSUS_MODULE_NAME;
    const STRUCT_NAME: &'static str = "EpochData";
}

#[derive(Debug)]
pub struct EpochInfo {
    start_number: u64,
    end_number: u64,
    block_time_target: u64,
    block_difficulty_window: u64,
}

impl EpochInfo {
    pub fn new(epoch: &EpochResource, consensus: &Consensus) -> Self {
        EpochInfo {
            start_number: epoch.start_number,
            end_number: epoch.end_number,
            block_time_target: epoch.block_time_target,
            block_difficulty_window: consensus.block_difficulty_window,
        }
    }

    pub fn start_number(&self) -> u64 {
        self.start_number
    }

    pub fn end_number(&self) -> u64 {
        self.end_number
    }

    pub fn block_time_target(&self) -> u64 {
        self.block_time_target
    }

    pub fn block_difficulty_window(&self) -> u64 {
        self.block_difficulty_window
    }
}
