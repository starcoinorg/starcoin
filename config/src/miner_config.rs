// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct MinerConfig {
    #[structopt(long = "disable-mint-empty-block")]
    /// Do not mint empty block, default is true in Dev network.
    pub disable_mint_empty_block: Option<bool>,
    #[structopt(long = "block-gas-limit")]
    pub block_gas_limit: Option<u64>,
    #[structopt(long = "disable-miner-client")]
    /// Don't start a miner client in node.
    pub disable_miner_client: Option<bool>,
    #[structopt(flatten)]
    pub client_config: MinerClientConfig,
}
impl Default for MinerConfig {
    fn default() -> Self {
        Self {
            disable_mint_empty_block: None,
            block_gas_limit: None,
            disable_miner_client: None,
            client_config: MinerClientConfig::default(),
        }
    }
}
impl MinerConfig {
    pub fn disable_miner_client(&self) -> bool {
        self.disable_miner_client.unwrap_or(false)
    }
    pub fn is_disable_mint_empty_block(&self) -> bool {
        self.disable_mint_empty_block.unwrap_or(false)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct MinerClientConfig {
    #[structopt(skip)]
    pub server: Option<String>,
    #[structopt(skip)]
    pub plugin_path: Option<String>,
    #[structopt(long = "miner-thread")]
    /// Miner thread number, not work for dev network, default is 1
    pub miner_thread: Option<u16>,
    #[structopt(long = "enable-stderr")]
    #[serde(skip)]
    pub enable_stderr: bool,
}
impl MinerClientConfig {
    pub fn miner_thread(&self) -> u16 {
        self.miner_thread.unwrap_or(1)
    }
}
impl Default for MinerClientConfig {
    fn default() -> Self {
        Self {
            server: None,
            plugin_path: None,
            miner_thread: Some(1),
            enable_stderr: false,
        }
    }
}
impl ConfigModule for MinerConfig {
    fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self> {
        // only dev network is on demand mine at default.
        let disable_mint_empty_block = opt
            .miner
            .disable_mint_empty_block
            .as_ref()
            .cloned()
            .unwrap_or_else(|| base.net.is_dev());
        Ok(Self {
            disable_mint_empty_block: Some(disable_mint_empty_block),
            block_gas_limit: None,
            disable_miner_client: opt.miner.disable_miner_client,
            client_config: MinerClientConfig {
                server: None,
                plugin_path: None,
                miner_thread: opt.miner.client_config.miner_thread,
                enable_stderr: false,
            },
        })
    }

    fn after_load(&mut self, opt: &StarcoinOpt, base: &BaseConfig) -> Result<()> {
        // only dev network is on demand mine at default.
        let disable_mint_empty_block = opt
            .miner
            .disable_mint_empty_block
            .as_ref()
            .cloned()
            .unwrap_or_else(|| base.net.is_dev());
        self.disable_mint_empty_block = Some(disable_mint_empty_block);
        if opt.miner.client_config.miner_thread.is_some() {
            self.client_config.miner_thread = opt.miner.client_config.miner_thread;
        }
        if opt.miner.disable_miner_client.is_some() {
            self.disable_miner_client = opt.miner.disable_miner_client;
        }
        Ok(())
    }
}
