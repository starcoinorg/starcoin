// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{register, Opts, PrometheusError, Registry, UIntCounterVec, UIntGauge};

#[derive(Clone)]
pub struct SyncMetrics {
    pub sync_task_total: UIntCounterVec,
    pub sync_task_break_total: UIntCounterVec,
    // TODO should move the task metrics to sync task.
    pub sync_block_count: UIntGauge,
    pub sync_peer_count: UIntGauge,
    pub sync_time: UIntGauge,
    pub sync_time_per_block: UIntGauge,
}

impl SyncMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let sync_task_total = UIntCounterVec::new(
            Opts::new(
                "sync_task_total",
                "sync counter, how many sync task for every type".to_string(),
            ),
            &["type"],
        )?;

        let sync_task_break_total = UIntCounterVec::new(
            Opts::new(
                "sync_task_break_total",
                "sync break counter, how many sync break for every type".to_string(),
            ),
            &["type"],
        )?;

        let sync_block_count = UIntGauge::with_opts(Opts::new(
            "sync_block_count",
            "how many block synced in latest sync task".to_string(),
        ))?;

        let sync_peer_count = UIntGauge::with_opts(Opts::new(
            "sync_peer_count",
            "how many peers used for latest sync task".to_string(),
        ))?;

        let sync_time = UIntGauge::with_opts(Opts::new(
            "sync_time",
            "how many milliseconds were used in latest sync task".to_string(),
        ))?;

        let sync_time_per_block = UIntGauge::with_opts(Opts::new(
            "sync_time_per_block",
            "how many milliseconds were used for sync one block in latest sync task.".to_string(),
        ))?;

        Ok(Self {
            sync_task_total: register(sync_task_total, registry)?,
            sync_task_break_total: register(sync_task_break_total, registry)?,
            sync_block_count: register(sync_block_count, registry)?,
            sync_peer_count: register(sync_peer_count, registry)?,
            sync_time: register(sync_time, registry)?,
            sync_time_per_block: register(sync_time_per_block, registry)?,
        })
    }
}
