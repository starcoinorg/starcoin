// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_port, BaseConfig, ChainNetwork, ConfigModule,
    StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
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
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self::default_with_net(ChainNetwork::default())
    }
}

impl ConfigModule for MinerConfig {
    fn default_with_net(net: ChainNetwork) -> Self {
        let port = match net {
            ChainNetwork::Dev => get_available_port_from(DEFAULT_STRATUM_SERVER_PORT),
            _ => DEFAULT_STRATUM_SERVER_PORT,
        };
        let block_gas_limit = net.block_gas_limit();
        Self {
            stratum_server: format!("127.0.0.1:{}", port)
                .parse::<SocketAddr>()
                .expect("parse address must success."),
            thread_num: 1,
            enable_miner_client: true,
            enable_mint_empty_block: true,
            enable_stderr: false,
            block_gas_limit,
        }
    }

    fn random(&mut self, _base: &BaseConfig) {
        self.stratum_server = format!("127.0.0.1:{}", get_random_available_port())
            .parse::<SocketAddr>()
            .unwrap();
        self.enable_mint_empty_block = true;
    }

    fn load(&mut self, base: &BaseConfig, opt: &StarcoinOpt) -> Result<()> {
        let disable_mint_empty_block = opt
            .disable_mint_empty_block
            .as_ref()
            .cloned()
            .unwrap_or_else(|| base.net.is_dev());

        if let Some(thread_num) = opt.miner_thread {
            self.thread_num = thread_num;
        }

        if opt.disable_miner_client {
            self.enable_miner_client = false;
        }
        self.enable_mint_empty_block = !disable_mint_empty_block;
        Ok(())
    }
}
