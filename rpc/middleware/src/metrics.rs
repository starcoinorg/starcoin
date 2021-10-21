// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use starcoin_metrics::{
    register, HistogramOpts, HistogramVec, Opts, PrometheusError, Registry, UIntCounterVec,
};

#[derive(Clone)]
pub struct RpcMetrics {
    pub rpc_counter: UIntCounterVec,
    pub rpc_timer: HistogramVec,
}

impl RpcMetrics {
    pub fn new_rpc_timer() -> Result<HistogramVec, PrometheusError> {
        HistogramVec::new(
            HistogramOpts::new("rpc_time", "Histogram of rpc request").namespace("starcoin"),
            &["method"],
        )
    }

    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let rpc_counter = register(
            UIntCounterVec::new(
                Opts::new("rpc", "Counters of how many rpc request").namespace("starcoin"),
                &["type", "method", "code"],
            )?,
            registry,
        )?;
        let rpc_timer = register(Self::new_rpc_timer()?, registry)?;
        Ok(Self {
            rpc_counter,
            rpc_timer,
        })
    }
}
