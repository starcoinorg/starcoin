// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{
    register, Histogram, HistogramOpts, Opts, PrometheusError, Registry, UIntCounter,
    UIntCounterVec, UIntGauge,
};

const SC_NS: &str = "starcoin";
const PREFIX: &str = "chain_";

#[derive(Clone)]
pub struct ChainMetrics {
    pub block_connect_counters: UIntCounterVec,
    pub block_connect_time: Histogram,
    pub rollback_block_counter: UIntCounter,
    pub block_num: UIntGauge,
    pub txn_num: UIntGauge,
}

impl ChainMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let block_connect_counters = register(
            UIntCounterVec::new(
                Opts::new(
                    format!("{}{}", PREFIX, "block_connect_counters"),
                    "block connect count".to_string(),
                )
                .namespace(SC_NS),
                &["type"],
            )?,
            registry,
        )?;

        let block_connect_time = register(
            Histogram::with_opts(
                HistogramOpts::new(
                    format!("{}{}", PREFIX, "block_connect_time"),
                    "connect block time".to_string(),
                )
                .namespace(SC_NS),
            )?,
            registry,
        )?;

        let rollback_block_counter = register(
            UIntCounter::with_opts(
                Opts::new(
                    format!("{}{}", PREFIX, "rollback_block_counter"),
                    "rollback block counter".to_string(),
                )
                .namespace(SC_NS),
            )?,
            registry,
        )?;

        let block_num = register(
            UIntGauge::with_opts(
                Opts::new(
                    format!("{}{}", PREFIX, "block_num"),
                    "how many block in main chain".to_string(),
                )
                .namespace(SC_NS),
            )?,
            registry,
        )?;

        let txn_num = register(
            UIntGauge::with_opts(
                Opts::new(
                    format!("{}{}", PREFIX, "txn_num"),
                    "how many txn in main chain".to_string(),
                )
                .namespace(SC_NS),
            )?,
            registry,
        )?;

        Ok(Self {
            block_connect_counters,
            block_connect_time,
            rollback_block_counter,
            block_num,
            txn_num,
        })
    }
}
