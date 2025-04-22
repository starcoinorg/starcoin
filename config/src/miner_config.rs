// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub static G_MAX_PARENTS_COUNT: u64 = 10;
pub static G_DAG_BLOCK_RECEIVE_TIME_WINDOW: u64 = 2; // in second, 2s for default
pub static G_MERGE_DEPTH: u64 = 3600; // the merge depth should be smaller than the pruning finality

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct MinerConfig {
    #[serde(skip)]
    #[clap(long = "disable-mint-empty-block")]
    /// Do not mint empty block, default is true in Dev network, only support cli.
    pub disable_mint_empty_block: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "miner-block-gas-limit")]
    /// Node local block_gas_limit, use min(config.block_gas_limit, onchain.block_gas_limit)
    pub block_gas_limit: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "disable-miner-client")]
    /// Don't start a miner client in node. The main network miner client is disable in default.
    /// This flag support both cli and config file.
    pub disable_miner_client: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "miner-thread")]
    /// Miner client thread number, not work for dev network, default is 1
    pub miner_thread: Option<u16>,

    #[serde(skip)]
    #[clap(skip)]
    base: Option<Arc<BaseConfig>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "maximum-parents-count")]
    pub maximum_parents_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "dag-block-receive-time-window")]
    pub dag_block_receive_time_window: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "dag-merge-depth")]
    pub dag_merge_depth: Option<u64>,
}

impl MinerConfig {
    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init")
    }
    pub fn disable_miner_client(&self) -> bool {
        //The main network miner client is disable in default.
        self.disable_miner_client
            .unwrap_or_else(|| self.base().net.is_main())
    }
    pub fn is_disable_mint_empty_block(&self) -> bool {
        self.disable_mint_empty_block
            .unwrap_or_else(|| self.base().net().is_dev())
    }
    pub fn miner_client_config(&self) -> Option<MinerClientConfig> {
        if self.disable_miner_client() {
            return None;
        }
        Some(MinerClientConfig {
            server: None,
            plugin_path: None,
            miner_thread: self.miner_thread.unwrap_or(1),
            enable_stderr: true,
        })
    }

    pub fn maximum_parents_count(&self) -> u64 {
        self.maximum_parents_count.unwrap_or(G_MAX_PARENTS_COUNT)
    }

    pub fn dag_block_receive_time_window(&self) -> u64 {
        self.dag_block_receive_time_window
            .unwrap_or(G_DAG_BLOCK_RECEIVE_TIME_WINDOW)
    }

    pub fn dag_merge_depth(&self) -> u64 {
        self.dag_merge_depth.unwrap_or(G_MERGE_DEPTH)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MinerClientConfig {
    pub server: Option<String>,
    pub plugin_path: Option<String>,
    pub miner_thread: u16,
    pub enable_stderr: bool,
}

impl MinerClientConfig {
    pub fn miner_thread(&self) -> u16 {
        self.miner_thread
    }
}

impl Default for MinerClientConfig {
    fn default() -> Self {
        Self {
            server: None,
            plugin_path: None,
            miner_thread: 1,
            enable_stderr: false,
        }
    }
}

impl ConfigModule for MinerConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);
        if opt.miner.miner_thread.is_some() {
            self.miner_thread = opt.miner.miner_thread;
        }
        if opt.miner.disable_miner_client.is_some() {
            self.disable_miner_client = opt.miner.disable_miner_client;
        }
        if opt.miner.disable_mint_empty_block.is_some() {
            self.disable_mint_empty_block = opt.miner.disable_mint_empty_block;
        }
        if opt.miner.block_gas_limit.is_some() {
            self.block_gas_limit = opt.miner.block_gas_limit;
        }

        if opt.miner.maximum_parents_count.is_some() {
            self.maximum_parents_count = opt.miner.maximum_parents_count;
        }

        if opt.miner.dag_block_receive_time_window.is_some() {
            self.dag_block_receive_time_window = opt.miner.dag_block_receive_time_window;
        }

        if opt.miner.dag_merge_depth.is_some() {
            self.dag_merge_depth = opt.miner.dag_merge_depth;
        }

        Ok(())
    }
}
