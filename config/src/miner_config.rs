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
    pub disable_mint_empty_block: Option<bool>,
    #[structopt(long = "block-gas-limit")]
    pub block_gas_limit: Option<u64>,
    #[structopt(long = "disable-miner-client")]
    pub disable_miner_client: bool,
    #[structopt(flatten)]
    pub client_config: MinerClientConfig,
}

impl MinerConfig {
    pub fn is_disable_mint_empty_block(&self) -> bool {
        if let Some(disable) = self.disable_mint_empty_block {
            disable
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct MinerClientConfig {
    pub server: Option<String>,
    pub plugin_path: Option<String>,
    #[structopt(long = "thread-num")]
    pub thread_num: u16,
    #[structopt(long = "enable-stderr")]
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
        Ok(Self {
            disable_mint_empty_block: Some(disable_mint_empty_block),
            block_gas_limit: None,
            disable_miner_client: opt.disable_miner_client,
            client_config: MinerClientConfig {
                server: None,
                plugin_path: None,
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
        self.disable_mint_empty_block = Some(disable_mint_empty_block);
        if let Some(thread) = opt.miner_thread {
            self.client_config.thread_num = thread;
        }
        if opt.disable_miner_client {
            self.disable_miner_client = true;
        }
        Ok(())
    }
}
