// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_port, BaseConfig, ChainNetwork, ConfigModule,
    StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MetricsConfig {
    pub enable_metrics: bool,
    pub metrics_server_port: u16,
    pub address: String,
}

pub static DEFAULT_METRIC_SERVER_PORT: u16 = 9101;

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            address: "0.0.0.0".to_string(),
            metrics_server_port: DEFAULT_METRIC_SERVER_PORT,
        }
    }
}

impl ConfigModule for MetricsConfig {
    fn default_with_net(net: ChainNetwork) -> Self {
        let mut config = Self::default();
        if net == ChainNetwork::Dev {
            config.metrics_server_port = get_available_port_from(DEFAULT_METRIC_SERVER_PORT);
        }
        config
    }

    fn random(&mut self, _base: &BaseConfig) {
        let port = get_random_available_port();
        self.metrics_server_port = port;
    }

    fn load(&mut self, _base: &BaseConfig, _opt: &StarcoinOpt) -> Result<()> {
        Ok(())
    }
}
