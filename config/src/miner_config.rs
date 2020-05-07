// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{get_available_port, BaseConfig, ChainNetwork, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub static DEFAULT_STRATUM_SERVER_PORT: u16 = 9940;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinerConfig {
    pub stratum_server: SocketAddr,
    /// Block period in second to use in dev network mode (0 = mine only if transaction pending)
    /// The real use time is a random value between 0 and dev_period.
    pub dev_period: u64,
    pub thread_num: u16,
    pub enable: bool,
    #[serde(skip)]
    pub pacemaker_strategy: PacemakerStrategy,
    #[serde(skip)]
    pub consensus_strategy: ConsensusStrategy,
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self::default_with_net(ChainNetwork::default())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum ConsensusStrategy {
    Argon(u16),
    Dummy(u64),
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
        let (pacemaker_strategy, consensus_strategy) = match net {
            ChainNetwork::Dev => (PacemakerStrategy::Ondemand, ConsensusStrategy::Dummy(0)),
            _ => (PacemakerStrategy::HeadBlock, ConsensusStrategy::Argon(1)),
        };
        let port = match net {
            ChainNetwork::Dev => get_available_port(),
            _ => DEFAULT_STRATUM_SERVER_PORT,
        };
        Self {
            stratum_server: format!("127.0.0.1:{}", port)
                .parse::<SocketAddr>()
                .expect("parse address must success."),
            dev_period: 0,
            thread_num: 1,
            enable: true,
            pacemaker_strategy,
            consensus_strategy,
        }
    }

    fn random(&mut self, _base: &BaseConfig) {
        self.stratum_server = format!("127.0.0.1:{}", get_available_port())
            .parse::<SocketAddr>()
            .unwrap();
        self.pacemaker_strategy = PacemakerStrategy::Schedule;
        self.consensus_strategy = ConsensusStrategy::Dummy(1);
    }

    fn load(&mut self, base: &BaseConfig, opt: &StarcoinOpt) -> Result<()> {
        if base.net.is_dev() && opt.dev_period > 0 {
            self.pacemaker_strategy = PacemakerStrategy::Schedule;
            self.consensus_strategy = ConsensusStrategy::Dummy(opt.dev_period);
        } else if !base.net.is_dev() {
            if let Some(thread_num) = opt.miner_thread {
                self.thread_num = thread_num;
            }
            self.pacemaker_strategy = PacemakerStrategy::HeadBlock;
            self.consensus_strategy = ConsensusStrategy::Argon(self.thread_num);
        }
        if let Some(enable) = opt.enable_mine {
            self.enable = enable;
        }
        Ok(())
    }
}
