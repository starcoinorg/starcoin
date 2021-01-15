// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_ports, parse_key_val, ApiQuotaConfig, ApiSet,
    BaseConfig, ConfigModule, QuotaDuration, StarcoinOpt,
};
use anyhow::Result;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

const DEFAULT_MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;
//10M
const DEFAULT_IPC_FILE: &str = "starcoin.ipc";
const DEFAULT_HTTP_PORT: u16 = 9850;
const DEFAULT_TCP_PORT: u16 = 9860;
const DEFAULT_WEB_SOCKET_PORT: u16 = 9870;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct HttpConfiguration {
    #[structopt(
        name = "disable-http-rpc",
        long,
        help = "disable http jsonrpc endpoint"
    )]
    pub disable: bool,
    #[structopt(
        name = "http-apis",
        long,
        default_value = "safe",
        help = "rpc apiset to serve"
    )]
    pub apis: ApiSet,
    /// Default http port is 9850
    #[structopt(name = "http-port", long)]
    pub port: Option<u16>,
    /// Default is 10M
    #[structopt(
        name = "http-max-request-body",
        long,
        help = "max request body in bytes"
    )]
    pub max_request_body_size: Option<usize>,
    #[structopt(name = "http-threads", long, help = "threads to use")]
    pub threads: Option<usize>,
    #[structopt(
        name = "http-ip-headers",
        long,
        use_delimiter = true,
        help = "list of http header which identify a ip",
        default_value = "X-Real-IP,X-Forwarded-For"
    )]
    pub ip_headers: Option<Vec<String>>,
}

impl Default for HttpConfiguration {
    fn default() -> Self {
        Self {
            disable: false,
            apis: ApiSet::UnsafeContext,
            max_request_body_size: None,
            threads: None,
            port: None,
            ip_headers: None,
        }
    }
}

impl HttpConfiguration {
    pub fn max_request_body_size(&self) -> usize {
        self.max_request_body_size
            .unwrap_or(DEFAULT_MAX_REQUEST_BODY_SIZE)
    }
    pub fn threads(&self) -> usize {
        self.threads.unwrap_or_else(num_cpus::get)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct TcpConfiguration {
    #[structopt(name = "disable-tcp-rpc", long, help = "disable tcp jsonrpc endpoint")]
    pub disable: bool,
    #[structopt(
        name = "tcp-apis",
        long,
        help = "rpc apiset to serve",
        default_value = "safe"
    )]
    pub apis: ApiSet,
    /// Default tcp port is 9860
    #[structopt(name = "tcp-port", long)]
    pub port: Option<u16>,
}

impl Default for TcpConfiguration {
    fn default() -> Self {
        Self {
            disable: false,
            apis: ApiSet::UnsafeContext,
            port: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct WsConfiguration {
    #[structopt(
        name = "disable-websocket-rpc",
        long,
        help = "disable websocket jsonrpc endpoint"
    )]
    pub disable: bool,
    #[structopt(
        name = "websocket-apis",
        long,
        help = "rpc apiset to serve",
        default_value = "pubsub"
    )]
    pub apis: ApiSet,
    /// Default websocket port is 9870
    #[structopt(name = "websocket-port", long)]
    pub port: Option<u16>,
    /// Default is 10M
    #[structopt(
        name = "websocket-max-request-body",
        long,
        help = "max request body in bytes"
    )]
    pub max_request_body_size: Option<usize>,
}

impl Default for WsConfiguration {
    fn default() -> Self {
        Self {
            disable: false,
            apis: ApiSet::PubSub,
            max_request_body_size: None,
            port: None,
        }
    }
}

impl WsConfiguration {
    pub fn max_request_body_size(&self) -> usize {
        self.max_request_body_size
            .unwrap_or(DEFAULT_MAX_REQUEST_BODY_SIZE)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct IpcConfiguration {
    #[structopt(name = "disable-ipc-rpc", long, help = "disable ipc jsonrpc endpoint")]
    pub disable: bool,
    #[structopt(
        name = "ipc-apis",
        long,
        help = "rpc apiset to serve",
        default_value = "ipc"
    )]
    pub apis: ApiSet,
    #[structopt(name = "ipc-file", long, help = "ipc file")]
    pub ipc_file_path: Option<PathBuf>,
}

impl Default for IpcConfiguration {
    fn default() -> Self {
        Self {
            disable: false,
            apis: ApiSet::IpcContext,
            ipc_file_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct ApiQuotaConfiguration {
    #[structopt(
        name = "default-global-jsonrpc-quota",
        long,
        help = "default api quota, eg: 1000/s",
        default_value = "1000/s"
    )]
    pub default_global_api_quota: ApiQuotaConfig,

    // number_of_values = 1 forces the user to repeat the -D option for each key-value pair:
    // my_program -D a=1 -D b=2
    #[structopt(
    name = "custom-global-jsonrpc-quota",
    long,
    help = "customize api quota, eg: node.info=100/s",
    number_of_values = 1,
    parse(try_from_str = parse_key_val)
    )]
    pub custom_global_api_quota: Vec<(String, ApiQuotaConfig)>,

    #[structopt(
        name = "default-user-jsonrpc-quota",
        long,
        help = "default api quota of user, eg: 1000/s",
        default_value = "1000/s"
    )]
    pub default_user_api_quota: ApiQuotaConfig,

    #[structopt(
    name = "custom-user-jsonrpc-quota",
    long,
    help = "customize api quota of user, eg: node.info=100/s",
    number_of_values = 1,
    parse(try_from_str = parse_key_val)
    )]
    pub custom_user_api_quota: Vec<(String, ApiQuotaConfig)>,
}

impl Default for ApiQuotaConfiguration {
    fn default() -> Self {
        Self {
            default_global_api_quota: ApiQuotaConfig {
                max_burst: NonZeroU32::new(1000).unwrap(),
                duration: QuotaDuration::Second,
            },
            custom_global_api_quota: vec![],
            default_user_api_quota: ApiQuotaConfig {
                max_burst: NonZeroU32::new(50).unwrap(),
                duration: QuotaDuration::Second,
            },
            custom_user_api_quota: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcConfig {
    #[serde(default)]
    pub api_quota: ApiQuotaConfiguration,
    #[serde(default)]
    pub tcp: TcpConfiguration,
    #[serde(default)]
    pub http: HttpConfiguration,
    #[serde(default)]
    pub ws: WsConfiguration,
    #[serde(default)]
    pub ipc: IpcConfiguration,
    pub rpc_address: IpAddr,

    pub block_query_max_range: u64,
}

#[derive(Clone)]
pub struct ListenAddress {
    pub protocol: &'static str,
    pub address: IpAddr,
    pub port: u16,
}

impl ListenAddress {
    pub fn new(protocol: &'static str, address: IpAddr, port: u16) -> Self {
        Self {
            protocol,
            address,
            port,
        }
    }
}

impl Into<SocketAddr> for ListenAddress {
    fn into(self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
}

impl std::fmt::Display for ListenAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}:{}", self.protocol, self.address, self.port)
    }
}

impl std::fmt::Debug for ListenAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl RpcConfig {
    pub fn get_ipc_file(&self) -> &Path {
        self.ipc
            .ipc_file_path
            .as_ref()
            .expect("config should init first.")
    }
    pub fn get_http_address(&self) -> Option<ListenAddress> {
        if self.http.disable {
            return None;
        }
        Some(ListenAddress::new(
            "http",
            self.rpc_address,
            self.http.port.unwrap_or(DEFAULT_HTTP_PORT),
        ))
    }
    pub fn get_tcp_address(&self) -> Option<ListenAddress> {
        if self.tcp.disable {
            return None;
        }
        Some(ListenAddress::new(
            "tcp",
            self.rpc_address,
            self.tcp.port.unwrap_or(DEFAULT_TCP_PORT),
        ))
    }

    pub fn get_ws_address(&self) -> Option<ListenAddress> {
        if self.ws.disable {
            return None;
        }
        Some(ListenAddress::new(
            "ws",
            self.rpc_address,
            self.ws.port.unwrap_or(DEFAULT_WEB_SOCKET_PORT),
        ))
    }
    #[cfg(not(windows))]
    pub fn get_ipc_file_by_base(base: &BaseConfig) -> PathBuf {
        base.data_dir().join(DEFAULT_IPC_FILE)
    }

    #[cfg(windows)]
    pub fn get_ipc_file_by_base(_base: &BaseConfig) -> PathBuf {
        PathBuf::from(r"\\.\pipe").join(DEFAULT_IPC_FILE)
    }
}

impl ConfigModule for RpcConfig {
    fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self> {
        let rpc_address: IpAddr = opt
            .rpc_address
            .clone()
            .unwrap_or_else(|| "0.0.0.0".to_string())
            .parse()?;

        let mut config = Self {
            ws: opt.ws.clone(),
            tcp: opt.tcp.clone(),
            http: opt.http.clone(),
            ipc: opt.ipc.clone(),
            api_quota: opt.api_quotas.clone(),
            rpc_address,
            block_query_max_range: opt.block_query_max_range.unwrap_or(128),
        };

        if base.net.is_test() {
            let ports = get_random_available_ports(3);
            config.http.port = Some(ports[0]);
            config.tcp.port = Some(ports[1]);
            config.ws.port = Some(ports[2]);
        } else if base.net.is_dev() {
            config.http.port = Some(get_available_port_from(DEFAULT_HTTP_PORT));
            config.tcp.port = Some(get_available_port_from(DEFAULT_TCP_PORT));
            config.ws.port = Some(get_available_port_from(DEFAULT_WEB_SOCKET_PORT));
        }

        if config.ipc.ipc_file_path.is_none() {
            config.ipc.ipc_file_path = Some(Self::get_ipc_file_by_base(base));
        }

        Ok(config)
    }

    fn after_load(&mut self, opt: &StarcoinOpt, _base: &BaseConfig) -> Result<()> {
        if let Some(rpc_address) = opt.rpc_address.clone() {
            self.rpc_address = rpc_address.parse()?;
        }
        if !opt.http.disable {
            self.http.disable = false;
            self.http.apis = opt.http.apis.clone();
            if opt.http.port.is_some() {
                self.http.port = opt.http.port;
            }
            if opt.http.max_request_body_size.is_some() {
                self.http.max_request_body_size = opt.http.max_request_body_size;
            }
            if opt.http.threads.is_some() {
                self.http.threads = opt.http.threads;
            }
            if opt.http.ip_headers.is_some() {
                self.http.ip_headers = opt.http.ip_headers.clone();
            }
        }
        info!("Http rpc address: {:?}", self.get_http_address());
        if !opt.tcp.disable {
            self.tcp.disable = false;
            self.tcp.apis = opt.tcp.apis.clone();
            if opt.tcp.port.is_some() {
                self.tcp.port = opt.tcp.port;
            }
        }
        info!("TCP rpc address: {:?}", self.get_tcp_address());
        if !opt.ipc.disable {
            self.ipc.apis = opt.ipc.apis.clone();
            if opt.ipc.ipc_file_path.is_some() {
                self.ipc.ipc_file_path = opt.ipc.ipc_file_path.clone();
            }
        }
        info!("Ipc file path: {:?}", self.ipc.ipc_file_path);
        if !opt.ws.disable {
            self.ws.disable = false;
            self.ws.apis = opt.ws.apis.clone();
            if opt.ws.port.is_some() {
                self.ws.port = opt.ws.port;
            }
            if opt.ws.max_request_body_size.is_some() {
                self.ws.max_request_body_size = opt.ws.max_request_body_size;
            }
        }
        info!("Websocket rpc address: {:?}", self.get_ws_address());

        // cli option override config file.
        if let Some(r) = opt.block_query_max_range {
            self.block_query_max_range = r;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc_config::{ApiQuotaConfig, QuotaDuration};

    #[test]
    fn test_api_quota_config() {
        let config = "1000/s".parse::<ApiQuotaConfig>().unwrap();
        assert_eq!(config.max_burst.get(), 1000u32);
        assert_eq!(config.duration, QuotaDuration::Second);
        assert_eq!("1000/s", config.to_string().as_str());
    }
}
