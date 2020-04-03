// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{get_available_port, BaseConfig, ChainNetwork, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinerConfig {
    //TODO remove dev_mode properties.
    pub dev_mode: bool,
    pub stratum_server: SocketAddr,
    /// Block period in second to use in dev network mode (0 = mine only if transaction pending)
    /// The real use time is a random value between 0 and dev_period.
    pub dev_period: u64,
    pub pacemaker_strategy: PacemakerStrategy,
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self::default_with_net(ChainNetwork::default())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum PacemakerStrategy {
    HeadBlock,
    Ondemand,
    Schedule,
}

impl ConfigModule for MinerConfig {
    fn default_with_net(net: ChainNetwork) -> Self {
        let pacemaker_strategy = match net {
            ChainNetwork::Dev => PacemakerStrategy::Ondemand,
            _ => PacemakerStrategy::HeadBlock,
        };
        Self {
            dev_mode: net.is_dev(),
            stratum_server: "127.0.0.1:9000".parse::<SocketAddr>().unwrap(),
            dev_period: 0,
            pacemaker_strategy,
        }
    }

    fn random(&mut self, _base: &BaseConfig) {
        self.dev_mode = true;
        self.stratum_server = format!("127.0.0.1:{}", get_available_port())
            .parse::<SocketAddr>()
            .unwrap();
        self.pacemaker_strategy = PacemakerStrategy::Schedule;
        self.dev_period = 1;
    }

    fn load(&mut self, base: &BaseConfig, opt: &StarcoinOpt) -> Result<()> {
        if base.net.is_dev() {
            if opt.dev_period > 0 {
                self.dev_period = opt.dev_period;
                self.pacemaker_strategy = PacemakerStrategy::Schedule;
            }
        }
        Ok(())
    }
}
