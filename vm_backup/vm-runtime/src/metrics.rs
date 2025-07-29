// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{
    register, Histogram, HistogramOpts, HistogramVec, Opts, PrometheusError, Registry,
    UIntCounterVec,
};

#[derive(Clone)]
pub struct VMMetrics {
    pub vm_txn_exe_total: UIntCounterVec,
    pub vm_txn_exe_time: HistogramVec,
    pub vm_txn_gas_usage: Histogram,
}

impl VMMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let vm_txn_exe_total = register(
            UIntCounterVec::new(
                Opts::new("vm_txn_exe_total", "total executed transaction"),
                &["type", "status"],
            )?,
            registry,
        )?;
        let vm_txn_exe_time = register(
            HistogramVec::new(
                HistogramOpts::new("vm_txn_exe_time", "vm transaction execution time usage"),
                &["type"],
            )?,
            registry,
        )?;
        let vm_txn_gas_usage = register(
            Histogram::with_opts(HistogramOpts::new(
                "vm_txn_gas_usage",
                "vm user transaction execution gas usage",
            ))?,
            registry,
        )?;
        Ok(Self {
            vm_txn_exe_total,
            vm_txn_exe_time,
            vm_txn_gas_usage,
        })
    }
}
