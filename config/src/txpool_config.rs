// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_system::get_free_mem_size;
use structopt::StructOpt;
pub const DEFAULT_MEM_SIZE: u64 = 128 * 1024 * 1024; // 128M

#[derive(Default, Clone, Debug, Eq, PartialEq, Deserialize, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct TxPoolConfig {
    #[structopt(name = "txpool-max-count", long)]
    /// Maximal number of transactions in the pool. default to 4096
    max_count: Option<u64>,
    #[structopt(name = "txpool-max-per-sender", long)]
    /// Maximal number of transactions from single sender. default to 128
    max_per_sender: Option<u64>,
    #[structopt(name = "txpool-max-mem-usage", long)]
    /// Maximal memory usage. Default to half of current free mem of system.
    max_mem_usage: Option<u64>,

    #[structopt(name = "txpool-tx-propagate-interval", long)]
    /// interval(s) of tx propagation timer. default to 2.
    tx_propagate_interval: Option<u64>,

    #[structopt(name = "txpool-min-tx-propagate", long)]
    /// interval(s) of tx propagation timer, default to 256.
    min_tx_to_propagate: Option<usize>,
    #[structopt(name = "txpool-propagate-for-blocks", long)]
    /// max blocks to propagate txns for.
    propagate_for_blocks: Option<u64>,
}

impl TxPoolConfig {
    pub fn set_max_count(&mut self, max_count: u64) {
        self.max_count = Some(max_count);
    }
    pub fn max_count(&self) -> u64 {
        self.max_count.clone().unwrap_or(4096)
    }
    pub fn max_per_sender(&self) -> u64 {
        self.max_per_sender.clone().unwrap_or(128)
    }
    pub fn max_mem_usage(&self) -> u64 {
        self.max_mem_usage.clone().unwrap_or(DEFAULT_MEM_SIZE)
    }
    pub fn tx_propagate_interval(&self) -> u64 {
        self.tx_propagate_interval.unwrap_or(2)
    }
    pub fn min_tx_to_propagate(&self) -> usize {
        self.min_tx_to_propagate.unwrap_or(256)
    }
    pub fn propagate_for_blocks(&self) -> u64 {
        self.propagate_for_blocks.unwrap_or(4)
    }
}

impl ConfigModule for TxPoolConfig {
    fn default_with_opt(opt: &StarcoinOpt, _base: &BaseConfig) -> Result<Self> {
        let mut txpool_config = opt.txpool.clone();
        txpool_config
            .max_mem_usage
            .get_or_insert_with(|| match get_free_mem_size() {
                Ok(free) => {
                    if free > 0 {
                        free / 2
                    } else {
                        DEFAULT_MEM_SIZE
                    }
                }
                Err(_) => DEFAULT_MEM_SIZE,
            });
        Ok(txpool_config)
    }
    fn after_load(&mut self, opt: &StarcoinOpt, _base: &BaseConfig) -> Result<()> {
        let txpool_opt = &opt.txpool;
        if let Some(m) = txpool_opt.max_mem_usage.as_ref() {
            self.max_mem_usage = Some(*m);
        }
        if let Some(m) = txpool_opt.max_count.as_ref() {
            self.max_count = Some(*m);
        }
        if let Some(m) = txpool_opt.max_mem_usage.as_ref() {
            self.max_mem_usage = Some(*m);
        }
        if let Some(m) = txpool_opt.tx_propagate_interval.as_ref() {
            self.tx_propagate_interval = Some(*m);
        }
        if let Some(m) = txpool_opt.min_tx_to_propagate.as_ref() {
            self.min_tx_to_propagate = Some(*m);
        }
        if let Some(m) = txpool_opt.propagate_for_blocks.as_ref() {
            self.propagate_for_blocks = Some(*m);
        }
        Ok(())
    }
}
