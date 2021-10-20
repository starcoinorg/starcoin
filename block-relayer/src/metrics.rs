// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{
    register, Histogram, HistogramOpts, Opts, PrometheusError, Registry, UIntCounter,
    UIntCounterVec,
};

#[derive(Clone)]
pub struct BlockRelayerMetrics {
    pub txns_filled: UIntCounterVec,
    pub txns_filled_time: Histogram,
    pub block_relay_time: Histogram,
    pub txns_filled_failed: UIntCounter,
}

impl BlockRelayerMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let txns_filled = register(
            UIntCounterVec::new(
                Opts::new(
                    "txns_filled",
                    "Count of block filled transactions from network|txpool|prefill",
                )
                .namespace("starcoin"),
                &["source"],
            )?,
            registry,
        )?;

        let txns_filled_time = register(
            Histogram::with_opts(
                HistogramOpts::new("txns_filled_time", "txns filled time").namespace("starcoin"),
            )?,
            registry,
        )?;
        let block_relay_time = register(
            Histogram::with_opts(
                HistogramOpts::new(
                    "block_relay_time",
                    "block relay time, measure the time usage in network",
                )
                .namespace("starcoin"),
            )?,
            registry,
        )?;

        let txns_filled_failed = register(
            UIntCounter::with_opts(
                Opts::new(
                    "txns_filled_failed",
                    "txns filled failed counter".to_string(),
                )
                .namespace("starcoin"),
            )?,
            registry,
        )?;

        Ok(Self {
            txns_filled,
            txns_filled_time,
            block_relay_time,
            txns_filled_failed,
        })
    }
}
