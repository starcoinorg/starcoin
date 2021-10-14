// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{register, Opts, PrometheusError, Registry, UIntCounterVec, UIntGauge};

#[derive(Clone)]
pub struct SyncMetrics {
    pub sync_times: UIntCounterVec,
    pub sync_break_times: UIntCounterVec,
    pub sync_block_count: UIntGauge,
    pub sync_peer_count: UIntGauge,
    pub sync_time: UIntGauge,
    pub sync_time_per_block: UIntGauge,
}

impl SyncMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let sync_times = UIntCounterVec::new(
            Opts::new(
                "sync_times",
                "sync times counter, how many sync task for every type".to_string(),
            )
            .namespace("starcoin"),
            &["type"],
        )?;

        let sync_break_times = UIntCounterVec::new(
            Opts::new(
                "sync_break_times",
                "sync break times counter, how many sync break for every type".to_string(),
            )
            .namespace("starcoin"),
            &["type"],
        )?;

        let sync_block_count = UIntGauge::with_opts(
            Opts::new(
                "sync_block_count",
                "sync blocks count gauge, how many block synced".to_string(),
            )
            .namespace("starcoin"),
        )?;

        let sync_peer_count = UIntGauge::with_opts(
            Opts::new(
                "sync_peer_count",
                "sync peers gauge, how many peers of sync target".to_string(),
            )
            .namespace("starcoin"),
        )?;

        let sync_time = UIntGauge::with_opts(
            Opts::new(
                "sync_time",
                "sync time gauge, how many milliseconds were used for sync".to_string(),
            )
            .namespace("starcoin"),
        )?;

        let sync_time_per_block = UIntGauge::with_opts(
            Opts::new(
                "sync_time_per_block",
                "sync time per block gauge, how many milliseconds were used for sync one block"
                    .to_string(),
            )
            .namespace("starcoin"),
        )?;

        Ok(Self {
            sync_times: register(sync_times, registry)?,
            sync_break_times: register(sync_break_times, registry)?,
            sync_block_count: register(sync_block_count, registry)?,
            sync_peer_count: register(sync_peer_count, registry)?,
            sync_time: register(sync_time, registry)?,
            sync_time_per_block: register(sync_time_per_block, registry)?,
        })
    }
}
