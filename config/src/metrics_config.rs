// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_port, BaseConfig, ConfigModule, StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct MetricsConfig {
    #[structopt(name = "disable-metrics", long, help = "disable metrics")]
    pub disable_metrics: bool,
    #[structopt(name = "address", long, help = "address", default_value = "0.0.0.0")]
    pub address: String,
    #[structopt(name = "metrics-port", long, default_value = "9101")]
    pub port: u16,
}

pub static DEFAULT_METRIC_SERVER_PORT: u16 = 9101;

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
            disable_metrics: opt.disable_metrics,
            address: "0.0.0.0".to_string(),
            port,
        })
    }

    fn after_load(&mut self, opt: &StarcoinOpt, _base: &BaseConfig) -> Result<()> {
        self.disable_metrics = opt.disable_metrics;
        Ok(())
    }
}
