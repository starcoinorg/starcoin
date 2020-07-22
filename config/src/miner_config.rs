// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_port, BaseConfig, ChainNetwork, ConfigModule,
    StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;

pub static DEFAULT_STRATUM_SERVER_PORT: u16 = 9940;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinerConfig {
    pub stratum_server: SocketAddr,
    pub thread_num: u16,
    pub enable_miner_client: bool,
    pub enable_mint_empty_block: bool,
    #[serde(skip)]
    pub enable_stderr: bool,
    pub block_gas_limit: u64,
    #[serde(skip)]
    pub consensus_strategy: ConsensusStrategy,
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self::default_with_net(ChainNetwork::default())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum ConsensusStrategy {
    Argon(u16),
    Dev,
    Dummy,
}

impl fmt::Display for ConsensusStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConsensusStrategy::Dummy => write!(f, "dummy"),
            ConsensusStrategy::Dev => write!(f, "dev"),
            ConsensusStrategy::Argon(_) => write!(f, "argon"),
        }
    }
}

impl ConfigModule for MinerConfig {
    fn default_with_net(net: ChainNetwork) -> Self {
        let consensus_strategy = match net {
            ChainNetwork::Dev => ConsensusStrategy::Dev,
            _ => ConsensusStrategy::Argon(1),
        };
        let port = match net {
            ChainNetwork::Dev => get_available_port_from(DEFAULT_STRATUM_SERVER_PORT),
            _ => DEFAULT_STRATUM_SERVER_PORT,
        };
        let block_gas_limit = match net {
            ChainNetwork::Dev => 1_000_000, // 100w
            _ => 10_000_000,                //1000w
        };
        Self {
            stratum_server: format!("127.0.0.1:{}", port)
                .parse::<SocketAddr>()
                .expect("parse address must success."),
            thread_num: 1,
            enable_miner_client: true,
            enable_mint_empty_block: true,
            enable_stderr: false,
            block_gas_limit,
            consensus_strategy,
        }
    }

    fn random(&mut self, _base: &BaseConfig) {
        self.stratum_server = format!("127.0.0.1:{}", get_random_available_port())
            .parse::<SocketAddr>()
            .unwrap();
        self.consensus_strategy = ConsensusStrategy::Dummy;
        self.enable_mint_empty_block = true;
    }

    fn load(&mut self, base: &BaseConfig, opt: &StarcoinOpt) -> Result<()> {
        let disable_mint_empty_block = opt
            .disable_mint_empty_block
            .as_ref()
            .cloned()
            .unwrap_or_else(|| base.net.is_dev());
        if base.net.is_dev() {
            self.consensus_strategy = ConsensusStrategy::Dev;
        } else {
            if let Some(thread_num) = opt.miner_thread {
                self.thread_num = thread_num;
            }
            self.consensus_strategy = ConsensusStrategy::Argon(self.thread_num);
        }

        if opt.disable_miner_client {
            self.enable_miner_client = false;
        }
        self.enable_mint_empty_block = !disable_mint_empty_block;
        Ok(())
    }
}
