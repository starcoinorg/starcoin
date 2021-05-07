// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::CORE_CODE_ADDRESS;
use crate::identifier::Identifier;
use crate::on_chain_config::OnChainConfig;
use move_core_types::language_storage::StructTag;
use move_core_types::language_storage::TypeTag;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const CONSENSUS_CONFIG_MODULE_NAME: &str = "ConsensusConfig";
pub static CONSENSUS_CONFIG_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new(CONSENSUS_CONFIG_MODULE_NAME).unwrap());

/// The Consensus on chain.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConsensusConfig {
    pub uncle_rate_target: u64,
    pub base_block_time_target: u64,
    pub base_reward_per_block: u128,
    pub base_reward_per_uncle_percent: u64,
    pub epoch_block_count: u64,
    pub base_block_difficulty_window: u64,
    pub min_block_time_target: u64,
    pub max_block_time_target: u64,
    pub base_max_uncles_per_block: u64,
    pub base_block_gas_limit: u64,
    pub strategy: u8,
}

impl OnChainConfig for ConsensusConfig {
    const MODULE_IDENTIFIER: &'static str = CONSENSUS_CONFIG_MODULE_NAME;
    const CONF_IDENTIFIER: &'static str = CONSENSUS_CONFIG_MODULE_NAME;
}

impl ConsensusConfig {
    pub fn type_tag() -> TypeTag {
        TypeTag::Struct(StructTag {
            address: CORE_CODE_ADDRESS,
            module: CONSENSUS_CONFIG_IDENTIFIER.clone(),
            name: CONSENSUS_CONFIG_IDENTIFIER.clone(),
            type_params: vec![],
        })
    }
}
pub fn consensus_config_type_tag() -> TypeTag {
    ConsensusConfig::type_tag()
}
