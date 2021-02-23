// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_port, BaseConfig, ConfigModule, StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use structopt::StructOpt;

pub static DEFAULT_METRIC_SERVER_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
pub static DEFAULT_METRIC_SERVER_PORT: u16 = 9101;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct MetricsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "disable-metrics", long, help = "disable metrics")]
    /// disable the metrics server, this flag support both cli and config.
    pub disable_metrics: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "metrics-address", long)]
    /// Metrics server listen address, default is 0.0.0.0
    pub address: Option<IpAddr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "metrics-port", long)]
    /// Metrics server port, default is 9101
    pub port: Option<u16>,

    #[serde(skip)]
    #[structopt(skip)]
    base: Option<Arc<BaseConfig>>,

    #[serde(skip)]
    #[structopt(skip)]
    metrics_address: Option<SocketAddr>,
}
impl MetricsConfig {
    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn disable_metrics(&self) -> bool {
        self.disable_metrics.unwrap_or(false)
    }

    pub fn metrics_address(&self) -> Option<SocketAddr> {
        self.metrics_address
    }

    fn generate_address(&mut self) {
        if !self.disable_metrics() {
            self.metrics_address = Some(SocketAddr::new(
                self.address.unwrap_or(DEFAULT_METRIC_SERVER_ADDRESS),
                self.port.unwrap_or_else(|| {
                    let base = self.base();
                    if base.net.is_test() {
                        get_random_available_port()
                    } else if base.net.is_dev() {
                        get_available_port_from(DEFAULT_METRIC_SERVER_PORT)
                    } else {
                        DEFAULT_METRIC_SERVER_PORT
                    }
                }),
            ));
        }
    }
}

impl ConfigModule for MetricsConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);

        if opt.metrics.disable_metrics.is_some() {
            self.disable_metrics = opt.metrics.disable_metrics;
        }
        if opt.metrics.address.is_some() {
            self.address = opt.metrics.address;
        }
        if opt.metrics.port.is_some() {
            self.port = opt.metrics.port;
        }
        self.generate_address();
        Ok(())
    }
}
