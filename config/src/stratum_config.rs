use crate::{
    get_available_port_from, get_random_available_port, BaseConfig, ConfigModule, StarcoinOpt,
    StructOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

const DEFAULT_STRATUM_PORT: u16 = 9880;
// UNSPECIFIED is 0.0.0.0
const DEFAULT_STRATUM_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct StratumConfig {
    #[serde(skip)]
    #[structopt(name = "disable-stratum", long, help = "disable stratum")]
    pub disable: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "stratum-port", long)]
    /// Default tcp port is 9880
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long = "stratum-address")]
    /// Stratum address, default is 0.0.0.0
    pub address: Option<IpAddr>,

    #[structopt(skip)]
    #[serde(skip)]
    base: Option<Arc<BaseConfig>>,
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
