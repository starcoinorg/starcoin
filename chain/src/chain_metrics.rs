// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use starcoin_metrics::{register_int_gauge, IntGauge, Opts, PrometheusError};

const PREFIX: &str = "chain_";

pub static CHAIN_METRICS: Lazy<ChainMetrics> = Lazy::new(|| ChainMetrics::register().unwrap());

#[derive(Clone)]
pub struct ChainMetrics {
    pub current_head_number: IntGauge,
}

impl ChainMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let current_head_number = register_int_gauge!(Opts::new(
            format!("{}{}", PREFIX, "current_head_number"),
            "current head number".to_string()
        )
        .namespace("starcoin"))?;
        Ok(Self {
            current_head_number,
        })
    }
}
