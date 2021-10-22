// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{register, HistogramOpts, PrometheusError, Registry};
use starcoin_metrics::{HistogramVec, Opts, UIntCounterVec};

#[derive(Clone)]
pub struct NetworkMetrics {
    pub broadcast_counters: UIntCounterVec,
    pub broadcast_duration: HistogramVec,
    pub broadcast_in_message_counters: UIntCounterVec,
}

impl NetworkMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let broadcast_counters = register(
            UIntCounterVec::new(
                Opts::new(
                    "broadcast_counters",
                    "network broadcast message counter by in|out and protocol",
                ),
                &["in_or_out", "notification_protocol"],
            )?,
            registry,
        )?;
        let broadcast_duration = register(
            HistogramVec::new(
                HistogramOpts::new(
                    "broadcast_duration",
                    "network broadcast message duration by protocol",
                ),
                &["notification_protocol"],
            )?,
            registry,
        )?;

        let broadcast_in_message_counters = register(
            UIntCounterVec::new(
                Opts::new(
                    "broadcast_in_message_counters",
                    "network broadcast receive message counters by known|unknown and protocol",
                ),
                &["known_or_unknown", "notification_protocol"],
            )?,
            registry,
        )?;

        Ok(Self {
            broadcast_counters,
            broadcast_duration,
            broadcast_in_message_counters,
        })
    }
}
