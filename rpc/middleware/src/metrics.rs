// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use starcoin_metrics::{
    register, HistogramOpts, HistogramVec, Opts, PrometheusError, Registry, UIntCounterVec,
};

#[derive(Clone)]
pub struct RpcMetrics {
    pub json_rpc_total: UIntCounterVec,
    pub json_rpc_time: HistogramVec,
}

impl RpcMetrics {
    pub fn new_rpc_timer() -> Result<HistogramVec, PrometheusError> {
        HistogramVec::new(
            HistogramOpts::new("json_rpc_time", "Histogram of rpc request"),
            &["method"],
        )
    }

    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let json_rpc_total = register(
            UIntCounterVec::new(
                Opts::new("json_rpc_total", "Counters of how many rpc request"),
                &["type", "method", "code"],
            )?,
            registry,
        )?;
        let json_rpc_time = register(Self::new_rpc_timer()?, registry)?;
        Ok(Self {
            json_rpc_total,
            json_rpc_time,
        })
    }
}
