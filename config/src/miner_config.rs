// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinerConfig {
    pub pacemaker_strategy: PacemakerStrategy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum PacemakerStrategy {
    HeadBlock,
    Ondemand,
    Schedule,
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self {
            pacemaker_strategy: PacemakerStrategy::Schedule,
        }
    }
}
