// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{register, HistogramOpts, PrometheusError, Registry};
use starcoin_metrics::{HistogramVec, Opts, UIntCounterVec};

#[derive(Clone)]
pub struct NetworkMetrics {
    pub network_broadcast_total: UIntCounterVec,
    pub network_broadcast_time: HistogramVec,
    pub network_broadcast_in_msg_total: UIntCounterVec,
}

impl NetworkMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let network_broadcast_total = register(
            UIntCounterVec::new(
                Opts::new(
                    "network_broadcast_total",
                    "network broadcast message counter by in|out and protocol",
                ),
                &["in_or_out", "notification_protocol"],
            )?,
            registry,
        )?;
        let network_broadcast_time = register(
            HistogramVec::new(
                HistogramOpts::new(
                    "network_broadcast_time",
                    "network broadcast message duration by protocol",
                ),
                &["notification_protocol"],
            )?,
            registry,
        )?;

        let network_broadcast_in_msg_total = register(
            UIntCounterVec::new(
                Opts::new(
                    "network_broadcast_in_msg_total",
                    "network broadcast receive message counters by known|unknown and protocol",
                ),
                &["known_or_unknown", "notification_protocol"],
            )?,
            registry,
        )?;

        Ok(Self {
            network_broadcast_total,
            network_broadcast_time,
            network_broadcast_in_msg_total,
        })
    }
}
