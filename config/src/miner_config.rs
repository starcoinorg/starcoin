// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::get_available_port;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinerConfig {
    pub stratum_server: SocketAddr,
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
            stratum_server: "127.0.0.1:9000".parse::<SocketAddr>().unwrap(),
            pacemaker_strategy: PacemakerStrategy::Schedule,
        }
    }
}

impl MinerConfig {
    pub fn random_for_test() -> Self {
        Self {
            stratum_server: format!("127.0.0.1:{}", get_available_port())
                .parse::<SocketAddr>()
                .unwrap(),
            pacemaker_strategy: PacemakerStrategy::Schedule,
        }
    }

    pub fn load(&mut self, data_dir: &PathBuf) -> Result<()> {
        Ok(())
    }
}
