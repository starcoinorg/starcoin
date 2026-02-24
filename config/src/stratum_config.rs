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

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_window_shares: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_confirmations: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_settlement_interval_secs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_batch_period_secs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_max_retained_shares: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_max_retained_candidates: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_database_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_ingest_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(skip)]
    pub pplns_settlement_enabled: Option<bool>,
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

#[derive(Debug, Clone)]
pub struct StratumPplnsConfig {
    pub enabled: bool,
    pub ingest_enabled: bool,
    pub settlement_enabled: bool,
    pub window_shares: u64,
    pub confirmations: u64,
    pub settlement_interval_secs: u64,
    pub batch_period_secs: u64,
    pub max_retained_shares: u64,
    pub max_retained_candidates: usize,
    pub database_url: Option<String>,
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
            max_invalid_shares: self.max_invalid_shares.unwrap_or(60),
            max_job_misses: self.max_job_misses.unwrap_or(60),
            max_stale_shares: self.max_stale_shares.unwrap_or(60),
            max_workers_per_account: self.max_workers_per_account.unwrap_or(1024),
        }
    }

    pub fn pplns(&self) -> StratumPplnsConfig {
        let enabled = self.pplns_enabled.unwrap_or(false);
        let ingest_enabled = enabled && self.pplns_ingest_enabled.unwrap_or(true);
        let settlement_enabled = enabled && self.pplns_settlement_enabled.unwrap_or(true);
        let window_shares = self.pplns_window_shares.unwrap_or(20_000).max(1);
        let max_retained_shares = self
            .pplns_max_retained_shares
            .unwrap_or(window_shares.saturating_mul(8).max(window_shares + 1_024))
            .max(window_shares);
        StratumPplnsConfig {
            enabled,
            ingest_enabled,
            settlement_enabled,
            window_shares,
            confirmations: self.pplns_confirmations.unwrap_or(6).max(1),
            settlement_interval_secs: self.pplns_settlement_interval_secs.unwrap_or(10).max(1),
            batch_period_secs: self.pplns_batch_period_secs.unwrap_or(3_600).max(60),
            max_retained_shares,
            max_retained_candidates: self.pplns_max_retained_candidates.unwrap_or(4_096).max(64),
            database_url: self.pplns_database_url.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pplns_mode_switches() {
        let mut config = StratumConfig::default();
        config.pplns_enabled = Some(false);
        config.pplns_ingest_enabled = Some(true);
        config.pplns_settlement_enabled = Some(true);
        let pplns = config.pplns();
        assert!(!pplns.enabled);
        assert!(!pplns.ingest_enabled);
        assert!(!pplns.settlement_enabled);
    }

    #[test]
    fn test_pplns_defaults_disabled() {
        let config = StratumConfig::default();
        let pplns = config.pplns();
        assert!(!pplns.enabled);
    }

    #[test]
    fn test_pplns_split_mode_flags() {
        let mut config = StratumConfig::default();
        config.pplns_enabled = Some(true);
        config.pplns_ingest_enabled = Some(true);
        config.pplns_settlement_enabled = Some(false);
        config.pplns_batch_period_secs = Some(7_200);
        config.pplns_database_url = Some("postgres://localhost:5432/starcoin".to_string());
        let pplns = config.pplns();
        assert!(pplns.enabled);
        assert!(pplns.ingest_enabled);
        assert!(!pplns.settlement_enabled);
        assert_eq!(pplns.batch_period_secs, 7_200);
        assert_eq!(
            pplns.database_url.as_deref(),
            Some("postgres://localhost:5432/starcoin")
        );
    }
}
