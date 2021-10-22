// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{
    register, Histogram, HistogramOpts, Opts, PrometheusError, Registry, UIntGauge,
};

#[derive(Clone)]
pub struct MinerMetrics {
    pub block_mint_count: UIntGauge,
    pub block_mint_time: Histogram,
}

impl MinerMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let block_mint_count = register(
            UIntGauge::with_opts(Opts::new("block_mint_count", "Count of block mint"))?,
            registry,
        )?;

        let block_mint_time = register(
            Histogram::with_opts(HistogramOpts::new(
                "block_mint_time",
                "Histogram of block mint",
            ))?,
            registry,
        )?;

        Ok(Self {
            block_mint_count,
            block_mint_time,
        })
    }
}
