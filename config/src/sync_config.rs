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

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "range-locate")]
    /// the range location will be used if it is true
    /// the range location is to find the common ancestor by log(n) time complexity
    pub range_locate: Option<bool>,

    /// watchdog interval for sync progress check (seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "sync-watchdog-interval",
        long,
        help = "sync watchdog interval in seconds, default 30."
    )]
    watchdog_interval_secs: Option<u64>,

    /// watchdog stall threshold before cancel/restart (seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "sync-watchdog-stall-secs",
        long,
        help = "sync watchdog stall threshold in seconds, default 900."
    )]
    watchdog_stall_secs: Option<u64>,

    /// timeout for parallel block execute (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(
        name = "sync-execute-timeout-ms",
        long,
        help = "sync execute timeout in milliseconds, default 300000."
    )]
    execute_timeout_ms: Option<u64>,
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

    pub fn range_locate(&self) -> bool {
        self.range_locate.unwrap_or(false)
    }

    pub fn watchdog_interval_secs(&self) -> u64 {
        match self.watchdog_interval_secs {
            Some(value) if value > 0 => value,
            _ => 30,
        }
    }

    pub fn watchdog_stall_secs(&self) -> u64 {
        match self.watchdog_stall_secs {
            Some(value) if value > 0 => value,
            _ => 15 * 60,
        }
    }

    pub fn execute_timeout_ms(&self) -> u64 {
        match self.execute_timeout_ms {
            Some(value) if value > 0 => value,
            _ => 300_000,
        }
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

        if opt.sync.range_locate.is_some() {
            self.range_locate = opt.sync.range_locate;
        }

        if opt.sync.watchdog_interval_secs.is_some() {
            self.watchdog_interval_secs = opt.sync.watchdog_interval_secs;
        }

        if opt.sync.watchdog_stall_secs.is_some() {
            self.watchdog_stall_secs = opt.sync.watchdog_stall_secs;
        }

        if opt.sync.execute_timeout_ms.is_some() {
            self.execute_timeout_ms = opt.sync.execute_timeout_ms;
        }

        Ok(())
    }
}
