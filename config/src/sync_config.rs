// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use network_api::PeerStrategy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct SyncConfig {
    /// peer select strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "peer-select-strategy",
        long,
        help = "peer select strategy, default random."
    )]
    peer_select_strategy: Option<PeerStrategy>,

    /// max retry times, then sync task will failed
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "max-retry-times",
        long,
        help = "max retry times once sync block failed, default 15."
    )]
    max_retry_times: Option<u64>,
}

impl SyncConfig {
    pub fn peer_select_strategy(&self) -> PeerStrategy {
        self.peer_select_strategy.unwrap_or_default()
    }

    pub fn max_retry_times(&self) -> u64 {
        self.max_retry_times.unwrap_or(15)
    }
}

impl ConfigModule for SyncConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, _base: Arc<BaseConfig>) -> Result<()> {
        if opt.sync.peer_select_strategy.is_some() {
            self.peer_select_strategy = opt.sync.peer_select_strategy;
        }

        if opt.sync.max_retry_times.is_some() {
            self.max_retry_times = opt.sync.max_retry_times;
        }

        Ok(())
    }
}
