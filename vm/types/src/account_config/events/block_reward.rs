// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_resource::MoveResource;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockRewardEvent {
    pub block_number: u64,
    pub block_reward: u128,
    pub gas_fees: u128,
    pub miner: AccountAddress,
}

impl BlockRewardEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for BlockRewardEvent {
    const MODULE_NAME: &'static str = "BlockReward";
    const STRUCT_NAME: &'static str = "BlockRewardEvent";
}
