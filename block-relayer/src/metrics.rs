// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{
    register, Histogram, HistogramOpts, Opts, PrometheusError, Registry, UIntCounter,
    UIntCounterVec,
};

#[derive(Clone)]
pub struct BlockRelayerMetrics {
    pub txns_filled_total: UIntCounterVec,
    pub txns_filled_time: Histogram,
    pub block_relay_time: Histogram,
    pub txns_filled_failed_total: UIntCounter,
}

impl BlockRelayerMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let txns_filled_total = register(
            UIntCounterVec::new(
                Opts::new(
                    "txns_filled_total",
                    "Count of block filled transactions from network|txpool|prefill",
                ),
                &["type"],
            )?,
            registry,
        )?;

        let txns_filled_time = register(
            Histogram::with_opts(HistogramOpts::new("txns_filled_time", "txns filled time"))?,
            registry,
        )?;
        let block_relay_time = register(
            Histogram::with_opts(HistogramOpts::new(
                "block_relay_time",
                "block relay time, measure the time usage in network",
            ))?,
            registry,
        )?;

        let txns_filled_failed_total = register(
            UIntCounter::with_opts(Opts::new(
                "txns_filled_failed_total",
                "txns filled failed counter".to_string(),
            ))?,
            registry,
        )?;

        Ok(Self {
            txns_filled_total,
            txns_filled_time,
            block_relay_time,
            txns_filled_failed_total,
        })
    }
}
