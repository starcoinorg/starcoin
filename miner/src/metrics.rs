// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use starcoin_metrics::{
    default_registry, register_histogram_vec, register_int_gauge, HistogramOpts, HistogramVec,
    IntGauge, Opts, PrometheusError, UIntCounter,
};

pub static MINER_METRICS: Lazy<MinerMetrics> = Lazy::new(|| MinerMetrics::register().unwrap());

#[derive(Clone)]
pub struct MinerMetrics {
    pub block_mint_count: IntGauge,
    pub block_mint_time: HistogramVec,
    pub maybe_uncle_count: UIntCounter,
}

impl MinerMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let block_mint_count =
            register_int_gauge!(
                Opts::new("block_mint_count", "Count of block mint").namespace("starcoin")
            )?;
        let block_mint_time = register_histogram_vec!(
            HistogramOpts::new("block_mint_time", "Histogram of block mint").namespace("starcoin"),
            &["mint_time"]
        )?;
        let maybe_uncle_count = UIntCounter::new(
            "starcoin_maybe_uncle_count",
            "maybe uncle count".to_string(),
        )?;
        default_registry().register(Box::new(maybe_uncle_count.clone()))?;

        Ok(Self {
            block_mint_count,
            block_mint_time,
            maybe_uncle_count,
        })
    }
}
