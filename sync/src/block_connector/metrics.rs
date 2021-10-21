// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{
    register, Histogram, HistogramOpts, Opts, PrometheusError, Registry, UIntCounter,
    UIntCounterVec, UIntGauge,
};

#[derive(Clone)]
pub struct ChainMetrics {
    pub chain_block_connect_counters: UIntCounterVec,
    pub chain_block_connect_time: Histogram,
    pub chain_rollback_block_counter: UIntCounter,
    pub chain_block_num: UIntGauge,
    pub chain_txn_num: UIntGauge,
}

impl ChainMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let chain_block_connect_counters = register(
            UIntCounterVec::new(
                Opts::new("chain_block_connect_counters", "block connect count"),
                &["type"],
            )?,
            registry,
        )?;

        let chain_block_connect_time = register(
            Histogram::with_opts(HistogramOpts::new(
                "chain_block_connect_time",
                "connect block time",
            ))?,
            registry,
        )?;

        let chain_rollback_block_counter = register(
            UIntCounter::with_opts(Opts::new(
                "chain_rollback_block_counter",
                "rollback block counter",
            ))?,
            registry,
        )?;

        let chain_block_num = register(
            UIntGauge::with_opts(Opts::new("chain_block_num", "how many block in main chain"))?,
            registry,
        )?;

        let chain_txn_num = register(
            UIntGauge::with_opts(Opts::new("chain_txn_num", "how many txn in main chain"))?,
            registry,
        )?;

        Ok(Self {
            chain_block_connect_counters,
            chain_block_connect_time,
            chain_rollback_block_counter,
            chain_block_num,
            chain_txn_num,
        })
    }
}
