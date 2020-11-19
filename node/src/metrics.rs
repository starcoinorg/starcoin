// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::_once_cell::sync::Lazy;
use starcoin_metrics::{register_histogram, Histogram, PrometheusError};

pub static NODE_METRICS: Lazy<NodeMetrics> = Lazy::new(|| NodeMetrics::register().unwrap());

#[derive(Clone)]
pub struct NodeMetrics {
    pub block_latency: Histogram,
}

impl NodeMetrics {
    pub fn register() -> Result<Self, PrometheusError> {
        let request_block_latency =
            register_histogram!("request_block_latency", "request_block_latency")?;

        Ok(Self {
            block_latency: request_block_latency,
        })
    }
}
