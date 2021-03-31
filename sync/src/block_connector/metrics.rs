// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use starcoin_metrics::{
    default_registry, register_histogram_vec, register_int_gauge, HistogramOpts, HistogramVec,
    IntGauge, Opts, PrometheusError, UIntCounterVec,
};

const SC_NS: &str = "starcoin";
const PREFIX: &str = "starcoin_write_block_chain_";

pub static WRITE_BLOCK_CHAIN_METRICS: Lazy<ChainMetrics> =
    Lazy::new(|| ChainMetrics::register().unwrap());

#[derive(Clone)]
pub struct ChainMetrics {
    pub block_connect_count: UIntCounterVec,
    pub exe_block_time: HistogramVec,
    pub rollback_block_size: IntGauge,
    pub current_head_number: IntGauge,
}

impl ChainMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let exe_block_time = register_histogram_vec!(
            HistogramOpts::new(
                format!("{}{}", PREFIX, "exe_block_time"),
                "execute block time".to_string()
            )
            .namespace(SC_NS),
            &["time"]
        )?;

        let rollback_block_size = register_int_gauge!(Opts::new(
            format!("{}{}", PREFIX, "rollback_block_size"),
            "rollback block size".to_string()
        )
        .namespace(SC_NS))?;

        let current_head_number = register_int_gauge!(Opts::new(
            format!("{}{}", PREFIX, "current_head_number"),
            "current head number".to_string()
        )
        .namespace(SC_NS))?;

        let block_connect_count = UIntCounterVec::new(
            Opts::new(
                format!("{}{}", PREFIX, "block_connect_count"),
                "block connect count".to_string(),
            )
            .namespace(SC_NS),
            &["type"],
        )?;

        default_registry().register(Box::new(block_connect_count.clone()))?;

        Ok(Self {
            exe_block_time,
            rollback_block_size,
            current_head_number,
            block_connect_count,
        })
    }
}
