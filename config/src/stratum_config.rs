use crate::{
    get_available_port_from, get_random_available_port, BaseConfig, ConfigModule, Parser,
    StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

const DEFAULT_STRATUM_PORT: u16 = 9880;
// UNSPECIFIED is 0.0.0.0
const DEFAULT_STRATUM_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, Parser)]
pub struct StratumConfig {
    #[serde(skip)]
    #[clap(name = "disable-stratum", long, help = "disable stratum")]
    pub disable: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "stratum-port", long)]
    /// Default tcp port is 9880
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "stratum-address")]
    /// Stratum address, default is 0.0.0.0
    pub address: Option<IpAddr>,

    #[clap(skip)]
    #[serde(skip)]
    base: Option<Arc<BaseConfig>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub share_dedup_window_secs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub stale_window_secs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub share_rate_window_secs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub max_shares_per_window: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub max_invalid_shares: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub max_job_misses: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub max_stale_shares: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub max_workers_per_account: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct StratumLimits {
    pub share_dedup_window_secs: u64,
    pub stale_window_secs: u64,
    pub share_rate_window_secs: u64,
    pub max_shares_per_window: u32,
    pub max_invalid_shares: u32,
    pub max_job_misses: u32,
    pub max_stale_shares: u32,
    pub max_workers_per_account: usize,
}

impl StratumConfig {
    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }
    pub fn get_address(&self) -> Option<SocketAddr> {
        if self.disable {
            return None;
        }
        let base = self.base();
        let address = self.address.unwrap_or(DEFAULT_STRATUM_ADDRESS).to_string();
        let port = self.port.unwrap_or_else(|| {
            if base.net().is_test() {
                get_random_available_port()
            } else if base.net().is_dev() {
                get_available_port_from(DEFAULT_STRATUM_PORT)
            } else {
                DEFAULT_STRATUM_PORT
            }
        });
        format!("{}:{}", address, port).parse::<SocketAddr>().ok()
    }

    pub fn limits(&self) -> StratumLimits {
        StratumLimits {
            share_dedup_window_secs: self.share_dedup_window_secs.unwrap_or(600),
            stale_window_secs: self.stale_window_secs.unwrap_or(120),
            share_rate_window_secs: self.share_rate_window_secs.unwrap_or(10),
            max_shares_per_window: self.max_shares_per_window.unwrap_or(200),
            max_invalid_shares: self.max_invalid_shares.unwrap_or(20),
            max_job_misses: self.max_job_misses.unwrap_or(20),
            max_stale_shares: self.max_stale_shares.unwrap_or(20),
            max_workers_per_account: self.max_workers_per_account.unwrap_or(1024),
        }
    }
}

impl ConfigModule for StratumConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);
        if opt.stratum.address.is_some() {
            self.address = opt.rpc.rpc_address;
        }
        if opt.stratum.disable {
            self.disable = true;
        }
        if opt.stratum.port.is_some() {
            self.port = opt.stratum.port;
        }
        info!(
            "Stratum listen address: {:?}, port:{:?}",
            self.address, self.port
        );
        Ok(())
    }
}
