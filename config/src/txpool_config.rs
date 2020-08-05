// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TxPoolConfig {
    /// Maximal number of transactions in the pool.
    pub max_count: u64,
    /// Maximal number of transactions from single sender.
    pub max_per_sender: u64,
    /// Maximal memory usage.
    pub max_mem_usage: u64,
}

impl ConfigModule for TxPoolConfig {
    fn default_with_opt(_opt: &StarcoinOpt, _base: &BaseConfig) -> Result<Self> {
        Ok(Self {
            max_count: 1024,
            max_per_sender: 16,
            max_mem_usage: 64 * 1024 * 1024, // 64M
        })
    }
}
