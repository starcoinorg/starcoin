// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use move_core_types::language_storage::TypeTag;
use serde::{Deserialize, Serialize};

/// Reward configuration
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RewardConfig {
    /// how many blocks delay reward distribution.
    pub reward_delay: u64,
}

impl OnChainConfig for RewardConfig {
    const MODULE_IDENTIFIER: &'static str = "block_reward_config";
    const TYPE_IDENTIFIER: &'static str = "RewardConfig";
}

impl RewardConfig {
    pub fn type_tag() -> TypeTag {
        TypeTag::Struct(Box::new(RewardConfig::struct_tag()))
    }
}
