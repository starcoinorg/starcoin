// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_port, BaseConfig, ConfigModule, StarcoinOpt,
};
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use starcoin_metrics::{get_metric_from_registry, Registry};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

pub static G_DEFAULT_METRIC_SERVER_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
pub static G_DEFAULT_METRIC_SERVER_PORT: u16 = 9101;
pub static G_DEFAULT_METRIC_PUSH_AUTH_PASSWORD: &str = "";

pub static G_DEFAULT_METRIC_NAMESPACE: &str = "starcoin";

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Eq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct PushParameterConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "push-server-url", long)]
    /// Metrics push server url
    pub push_server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "auth-username", long)]
    /// Metrics push server auth username
    pub auth_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "auth-password", long)]
    /// Metrics push server auth password
    pub auth_password: Option<String>,
    #[clap(name = "push-interval", long, default_value = "5")]
    pub interval: u64,
}
impl PushParameterConfig {
    pub fn is_config(&self) -> bool {
        self.push_server_url.is_some()
    }
    pub fn push_server_url(&self) -> String {
        self.push_server_url.clone().unwrap()
    }
    pub fn interval(&self) -> u64 {
        self.interval
    }
    pub fn auth_username(&self) -> Option<String> {
        self.auth_username.clone()
    }
    pub fn auth_password(&self) -> String {
        self.auth_password
            .clone()
            .unwrap_or_else(|| G_DEFAULT_METRIC_PUSH_AUTH_PASSWORD.to_owned())
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct MetricsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "disable-metrics", long, help = "disable metrics")]
    /// disable the metrics server, this flag support both cli and config.
    pub disable_metrics: Option<bool>,

    #[serde(default)]
    #[clap(flatten)]
    /// Metrics push server parameter
    pub push_config: PushParameterConfig,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "metrics-address", long)]
    /// Metrics server listen address, default is 0.0.0.0
    pub address: Option<IpAddr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "metrics-port", long)]
    /// Metrics server port, default is 9101
    pub port: Option<u16>,

    #[serde(skip)]
    #[clap(skip)]
    base: Option<Arc<BaseConfig>>,

    #[serde(skip)]
    #[clap(skip)]
    metrics_address: Option<SocketAddr>,

    #[serde(skip)]
    #[clap(skip)]
    registry: Option<Registry>,
}

impl PartialEq for MetricsConfig {
    fn eq(&self, other: &Self) -> bool {
        (
            &self.disable_metrics,
            &self.push_config,
            &self.address,
            &self.port,
        ) == (
            &other.disable_metrics,
            &other.push_config,
            &other.address,
            &other.port,
        )
    }
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
                self.address.unwrap_or(G_DEFAULT_METRIC_SERVER_ADDRESS),
                self.port.unwrap_or_else(|| {
                    let base = self.base();
                    if base.net.is_test() || base.net.is_dag_test() {
                        get_random_available_port()
                    } else if base.net.is_dev() {
                        get_available_port_from(G_DEFAULT_METRIC_SERVER_PORT)
                    } else {
                        G_DEFAULT_METRIC_SERVER_PORT
                    }
                }),
            ));
        }
    }

    pub fn registry(&self) -> Option<&Registry> {
        self.registry.as_ref()
    }

    // this function just for test the metric.
    pub fn get_metric(
        &self,
        name: &str,
        label: Option<(&str, &str)>,
    ) -> Option<Vec<starcoin_metrics::proto::Metric>> {
        //auto add namespace
        let metric_name = if name.starts_with(G_DEFAULT_METRIC_NAMESPACE) {
            name.to_string()
        } else {
            format!("{}_{}", G_DEFAULT_METRIC_NAMESPACE, name)
        };
        self.registry
            .as_ref()
            .and_then(|registry| get_metric_from_registry(registry, metric_name.as_str(), label))
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
        if opt.metrics.push_config.is_config() {
            self.push_config = opt.metrics.push_config.clone();
        }
        self.generate_address();

        if !self.disable_metrics() {
            let registry =
                Registry::new_custom(Some(G_DEFAULT_METRIC_NAMESPACE.to_string()), None)?;
            self.registry = Some(registry);
        }

        Ok(())
    }
}
