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
#[serde(deny_unknown_fields)]
pub struct MinerConfig {
    pub stratum_server: SocketAddr,
    pub enable_mint_empty_block: bool,
    pub block_gas_limit: Option<u64>,
    pub enable_miner_client: bool,
    pub client_config: MinerClientConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MinerClientConfig {
    pub stratum_server: SocketAddr,
    pub thread_num: u16,
    #[serde(skip)]
    pub enable_stderr: bool,
}

impl ConfigModule for MinerConfig {
    fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self> {
        // only dev network is on demand mine at default.
        let disable_mint_empty_block = opt
            .disable_mint_empty_block
            .as_ref()
            .cloned()
            .unwrap_or_else(|| base.net.is_dev());

        let port = match base.net {
            ChainNetwork::Test => get_random_available_port(),
            ChainNetwork::Dev => get_available_port_from(DEFAULT_STRATUM_SERVER_PORT),
            _ => DEFAULT_STRATUM_SERVER_PORT,
        };
        let stratum_server = format!("127.0.0.1:{}", port).parse::<SocketAddr>()?;

        Ok(Self {
            stratum_server,
            enable_mint_empty_block: !disable_mint_empty_block,
            block_gas_limit: None,
            enable_miner_client: !opt.disable_miner_client,
            client_config: MinerClientConfig {
                stratum_server,
                thread_num: opt.miner_thread.unwrap_or(1),
                enable_stderr: false,
            },
        })
    }

    fn after_load(&mut self, opt: &StarcoinOpt, base: &BaseConfig) -> Result<()> {
        // only dev network is on demand mine at default.
        let disable_mint_empty_block = opt
            .disable_mint_empty_block
            .as_ref()
            .cloned()
            .unwrap_or_else(|| base.net.is_dev());

        self.enable_mint_empty_block = !disable_mint_empty_block;

        if let Some(thread) = opt.miner_thread {
            self.client_config.thread_num = thread;
        }
        if opt.disable_miner_client {
            self.enable_miner_client = false;
        }
        Ok(())
    }
}
