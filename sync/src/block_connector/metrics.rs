// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{
    register, Histogram, HistogramOpts, Opts, PrometheusError, Registry, UIntCounter,
    UIntCounterVec, UIntGauge,
};

#[derive(Clone)]
pub struct ChainMetrics {
    pub chain_block_connect_total: UIntCounterVec,
    pub chain_select_head_total: UIntCounterVec,
    pub chain_block_connect_time: Histogram,
    pub chain_rollback_block_total: UIntCounter,
    pub chain_block_num: UIntGauge,
    pub chain_txn_num: UIntGauge,
}

impl ChainMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let chain_block_connect_total = register(
            UIntCounterVec::new(
                Opts::new(
                    "chain_block_connect_total",
                    "total block try to connect to chain",
                ),
                &["type"],
            )?,
            registry,
        )?;

        let chain_select_head_total = register(
            UIntCounterVec::new(
                Opts::new(
                    "chain_select_head_total",
                    "total select head count, new_head or new_branch",
                ),
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

        let chain_rollback_block_total = register(
            UIntCounter::with_opts(Opts::new(
                "chain_rollback_block_total",
                "total rollback blocks",
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
            chain_block_connect_total,
            chain_select_head_total,
            chain_block_connect_time,
            chain_rollback_block_total,
            chain_block_num,
            chain_txn_num,
        })
    }
}
