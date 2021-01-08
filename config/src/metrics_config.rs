// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_port, BaseConfig, ConfigModule, StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub static DEFAULT_METRIC_SERVER_ADDRESS: &str = "0.0.0.0";
pub static DEFAULT_METRIC_SERVER_PORT: u16 = 9101;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct MetricsConfig {
    #[structopt(name = "disable-metrics", long, help = "disable metrics")]
    pub disable_metrics: Option<bool>,
    #[structopt(
        name = "address",
        long,
        help = "address",
        default_value = "DEFAULT_METRIC_SERVER_ADDRESS"
    )]
    pub address: String,
    #[structopt(name = "metrics-port", long, default_value = "9101")]
    pub port: u16,
}
impl MetricsConfig {
    pub fn disable_metrics(&self) -> bool {
        self.disable_metrics.unwrap_or(false)
    }
}
impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            disable_metrics: None,
            address: DEFAULT_METRIC_SERVER_ADDRESS.to_string(),
            port: DEFAULT_METRIC_SERVER_PORT,
        }
    }
}
impl ConfigModule for MetricsConfig {
    fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self> {
        let port = if base.net.is_test() {
            get_random_available_port()
        } else if base.net.is_dev() {
            get_available_port_from(DEFAULT_METRIC_SERVER_PORT)
        } else {
            DEFAULT_METRIC_SERVER_PORT
        };
        Ok(Self {
            disable_metrics: opt.metrics.disable_metrics,
            address: DEFAULT_METRIC_SERVER_ADDRESS.to_string(),
            port,
        })
    }

    fn after_load(&mut self, opt: &StarcoinOpt, _base: &BaseConfig) -> Result<()> {
        if opt.metrics.disable_metrics.is_some() {
            self.disable_metrics = opt.metrics.disable_metrics;
        }
        Ok(())
    }
}
