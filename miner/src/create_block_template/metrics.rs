// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_metrics::{register, Opts, PrometheusError, Registry, UIntGauge};

#[derive(Clone)]
pub struct BlockBuilderMetrics {
    pub current_epoch_maybe_uncles: UIntGauge,
}

impl BlockBuilderMetrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        let current_epoch_maybe_uncles = register(
            UIntGauge::with_opts(Opts::new(
                "current_epoch_maybe_uncles",
                "maybe uncle count in current epoch.",
            ))?,
            registry,
        )?;

        Ok(Self {
            current_epoch_maybe_uncles,
        })
    }
}
