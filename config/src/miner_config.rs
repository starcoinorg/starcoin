// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinerConfig {
    pub pacemaker_strategy: PacemakerStrategy,
    pub miner_address: Option<SocketAddr>,
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
            miner_address: Some("127.0.0.1:9000".parse::<SocketAddr>().unwrap()),
        }
    }
}
