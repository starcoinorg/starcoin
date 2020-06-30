// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

/// Defines the version of Starcoin software.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Consensus {
    pub uncle_rate_target: u64,
    pub epoch_time_target: u64,
    pub reward_half_time_target: u64,
}

impl OnChainConfig for Consensus {
    const IDENTIFIER: &'static str = "Consensus";
}
