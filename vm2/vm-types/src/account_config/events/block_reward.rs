// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
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

impl MoveStructType for BlockRewardEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("block_reward");
    const STRUCT_NAME: &'static IdentStr = ident_str!("BlockRewardEvent");
}

impl MoveResource for BlockRewardEvent {}
