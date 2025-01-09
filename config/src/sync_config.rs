// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use clap::Parser;
use network_api::PeerStrategy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Eq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct SyncConfig {
    /// peer select strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "peer-select-strategy",
        long,
        help = "peer select strategy, default random."
    )]
    peer_select_strategy: Option<PeerStrategy>,

    /// max retry times, then sync task will failed
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "max-retry-times",
        long,
        help = "max retry times once sync block failed, default 15."
    )]
    max_retry_times: Option<u64>,

    /// the maximum gap between the current head block's number and the peer's block's number
    /// and if the block height broadcast by a peer node is greater than the height of the local head block by this maximum value,
    /// a regular sync process will be initiated;
    /// otherwise, a lightweight sync process will be triggered, strengthening the reference relationship between nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "lightweight-sync-max-gap",
        long,
        help = "The height difference threshold for triggering a lightweight sync."
    )]
    lightweight_sync_max_gap: Option<u64>,
}

impl SyncConfig {
    pub fn peer_select_strategy(&self) -> PeerStrategy {
        self.peer_select_strategy.unwrap_or_default()
    }

    pub fn max_retry_times(&self) -> u64 {
        self.max_retry_times.unwrap_or(15)
    }

    pub fn lightweight_sync_max_gap(&self) -> Option<u64> {
        self.lightweight_sync_max_gap
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

        if opt.sync.lightweight_sync_max_gap.is_some() {
            self.lightweight_sync_max_gap = opt.sync.lightweight_sync_max_gap;
        }

        Ok(())
    }
}
