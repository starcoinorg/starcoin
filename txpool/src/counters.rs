// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{
    register, HistogramOpts, HistogramVec, Opts, PrometheusError, Registry, UIntCounterVec,
    UIntGaugeVec,
};

#[derive(Clone)]
pub struct TxPoolMetrics {
    pub txpool_txn_event_counter: UIntCounterVec,
    pub txpool_status: UIntGaugeVec,
    pub txpool_service_timer: HistogramVec,
}

impl TxPoolMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let txpool_txn_event_counter = UIntCounterVec::new(
            Opts::new(
                "txpool_txn_event_counter",
                "Counters of txn events, such as added|dropped|rejected etc",
            ),
            &["event"],
        )?;
        let txpool_status = UIntGaugeVec::new(
            Opts::new("txpool_status", "Gauge of pool status"),
            &["name"],
        )?;
        let txpool_service_timer = HistogramVec::new(
            HistogramOpts::new("txpool_service_timer", "Histogram of txpool service"),
            &["api"],
        )?;
        Ok(Self {
            txpool_txn_event_counter: register(txpool_txn_event_counter, registry)?,
            txpool_status: register(txpool_status, registry)?,
            txpool_service_timer: register(txpool_service_timer, registry)?,
        })
    }
}
