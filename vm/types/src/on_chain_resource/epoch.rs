// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::access_path::{AccessPath, DataPath};
use crate::event::EventHandle;
use crate::genesis_config::ConsensusStrategy;
use crate::move_resource::MoveResource;
use move_core_types::language_storage::{StructTag, CORE_CODE_ADDRESS};
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
/// The Epoch resource held under an account.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Epoch {
    number: u64,
    //milli_seconds
    start_time: u64,
    start_block_number: u64,
    end_block_number: u64,
    //milli_seconds
    block_time_target: u64,
    reward_per_block: u128,
    reward_per_uncle_percent: u64,
    block_difficulty_window: u64,
    max_uncles_per_block: u64,
    block_gas_limit: u64,
    strategy: u8,
    new_epoch_events: EventHandle,
}

impl Epoch {
    pub fn new(
        number: u64,
        start_time: u64,
        start_block_number: u64,
        end_block_number: u64,
        block_time_target: u64,
        reward_per_block: u128,
        reward_per_uncle_percent: u64,
        block_difficulty_window: u64,
        max_uncles_per_block: u64,
        block_gas_limit: u64,
        strategy: u8,
        new_epoch_events: EventHandle,
    ) -> Self {
        Self {
            number,
            start_time,
            start_block_number,
            end_block_number,
            block_time_target,
            reward_per_block,
            reward_per_uncle_percent,
            block_difficulty_window,
            max_uncles_per_block,
            block_gas_limit,
            strategy,
            new_epoch_events,
        }
    }

    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn start_time(&self) -> u64 {
        self.start_time
    }

    pub fn start_block_number(&self) -> u64 {
        self.start_block_number
    }

    pub fn end_block_number(&self) -> u64 {
        self.end_block_number
    }

    pub fn block_time_target(&self) -> u64 {
        self.block_time_target
    }

    pub fn reward_per_block(&self) -> u128 {
        self.reward_per_block
    }

    pub fn reward_per_uncle_percent(&self) -> u64 {
        self.reward_per_uncle_percent
    }

    pub fn block_difficulty_window(&self) -> u64 {
        self.block_difficulty_window
    }

    pub fn max_uncles_per_block(&self) -> u64 {
        self.max_uncles_per_block
    }

    pub fn block_gas_limit(&self) -> u64 {
        self.block_gas_limit
    }

    pub fn strategy(&self) -> ConsensusStrategy {
        ConsensusStrategy::try_from(self.strategy).expect("epoch consensus strategy must exist.")
    }

    // TODO/XXX: remove this once the MoveResource trait allows type arguments to `struct_tag`.
    pub fn struct_tag_for_epoch() -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: Epoch::struct_identifier(),
            module: Epoch::module_identifier(),
            type_params: vec![],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn data_path_for() -> DataPath {
        AccessPath::resource_data_path(Epoch::struct_tag_for_epoch())
    }
}

impl MoveResource for Epoch {
    const MODULE_NAME: &'static str = "Epoch";
    const STRUCT_NAME: &'static str = "Epoch";
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EpochInfo {
    epoch: Epoch,
    epoch_data: EpochData,
}

impl EpochInfo {
    pub fn new(epoch: Epoch, epoch_data: EpochData) -> Self {
        EpochInfo { epoch, epoch_data }
    }

    pub fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    pub fn epoch_data(&self) -> &EpochData {
        &self.epoch_data
    }

    pub fn start_block_number(&self) -> u64 {
        self.epoch.start_block_number
    }

    pub fn start_time(&self) -> u64 {
        self.epoch.start_time
    }

    pub fn end_block_number(&self) -> u64 {
        self.epoch.end_block_number
    }

    pub fn block_time_target(&self) -> u64 {
        self.epoch.block_time_target
    }

    pub fn block_difficulty_window(&self) -> u64 {
        self.epoch.block_difficulty_window
    }

    pub fn uncles(&self) -> u64 {
        self.epoch_data.uncles
    }

    pub fn total_reward(&self) -> u128 {
        self.epoch_data.total_reward
    }
    pub fn reward_per_block(&self) -> u128 {
        self.epoch.reward_per_block
    }
    pub fn reward_per_uncle_percent(&self) -> u64 {
        self.epoch.reward_per_uncle_percent
    }
    pub fn number(&self) -> u64 {
        self.epoch.number()
    }
}

/// The Epoch data resource held under an account.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EpochData {
    uncles: u64,
    total_reward: u128,
    total_gas: u128,
}

impl MoveResource for EpochData {
    const MODULE_NAME: &'static str = "Epoch";
    const STRUCT_NAME: &'static str = "EpochData";
}

impl EpochData {
    pub fn new(uncles: u64, total_reward: u128, total_gas: u128) -> Self {
        Self {
            uncles,
            total_reward,
            total_gas,
        }
    }

    pub fn uncles(&self) -> u64 {
        self.uncles
    }

    pub fn total_gas(&self) -> u128 {
        self.total_gas
    }

    pub fn total_reward(&self) -> u128 {
        self.total_reward
    }

    // TODO/XXX: remove this once the MoveResource trait allows type arguments to `struct_tag`.
    pub fn struct_tag_for_epoch() -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: EpochData::struct_identifier(),
            module: EpochData::module_identifier(),
            type_params: vec![],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn data_path_for() -> DataPath {
        AccessPath::resource_data_path(EpochData::struct_tag_for_epoch())
    }
}
