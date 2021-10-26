// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{
    register, HistogramOpts, HistogramVec, Opts, PrometheusError, Registry, UIntCounterVec,
    UIntGaugeVec,
};

#[derive(Clone)]
pub struct TxPoolMetrics {
    pub txpool_txn_event_total: UIntCounterVec,
    pub txpool_status: UIntGaugeVec,
    pub txpool_service_time: HistogramVec,
}

impl TxPoolMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let txpool_txn_event_total = register(
            UIntCounterVec::new(
                Opts::new(
                    "txpool_txn_event_total",
                    "Counters of txn events, such as added|dropped|rejected etc",
                ),
                &["type"],
            )?,
            registry,
        )?;
        let txpool_status = register(
            UIntGaugeVec::new(
                Opts::new("txpool_status", "Gauge of pool status"),
                &["name"],
            )?,
            registry,
        )?;
        let txpool_service_time = register(
            HistogramVec::new(
                HistogramOpts::new("txpool_service_time", "txpool service method time usage."),
                &["api"],
            )?,
            registry,
        )?;
        Ok(Self {
            txpool_txn_event_total,
            txpool_status,
            txpool_service_time,
        })
    }
}
