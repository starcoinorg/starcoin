// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_system::get_free_mem_size;
use structopt::StructOpt;
pub const DEFAULT_MEM_SIZE: u64 = 128 * 1024 * 1024; // 128M

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct TxPoolConfig {
    #[structopt(name = "max-count", long, default_value = "4096")]
    /// Maximal number of transactions in the pool.
    pub max_count: u64,
    #[structopt(name = "max-per-sender", long, default_value = "128")]
    /// Maximal number of transactions from single sender.
    pub max_per_sender: u64,
    #[structopt(name = "max-mem-usage", long, default_value = "134217728")]
    /// Maximal memory usage.
    pub max_mem_usage: u64,
}

impl ConfigModule for TxPoolConfig {
    fn default_with_opt(_opt: &StarcoinOpt, _base: &BaseConfig) -> Result<Self> {
        let free_mem = match get_free_mem_size() {
            Ok(free) => {
                if free > 0 {
                    free / 2
                } else {
                    DEFAULT_MEM_SIZE
                }
            }
            Err(_) => DEFAULT_MEM_SIZE,
        };
        Ok(Self {
            max_count: 4096,
            max_per_sender: 128,
            max_mem_usage: free_mem,
        })
    }
}
