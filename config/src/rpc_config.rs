// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_ports, parse_key_val, ApiQuotaConfig, ApiSet,
    BaseConfig, ConfigModule, QuotaDuration, StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use std::collections::HashSet;
use std::fmt::Formatter;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

//10M
const DEFAULT_MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;
const DEFAULT_IPC_FILE: &str = "starcoin.ipc";
const DEFAULT_HTTP_PORT: u16 = 9850;
const DEFAULT_TCP_PORT: u16 = 9860;
const DEFAULT_WEB_SOCKET_PORT: u16 = 9870;
// UNSPECIFIED is 0.0.0.0
const DEFAULT_RPC_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
const DEFAULT_BLOCK_QUERY_MAX_RANGE: u64 = 32;
const DEFAULT_TXN_INFO_QUEYR_MAX_RANGE: u64 = 32;

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct HttpConfiguration {
    #[serde(skip)]
    #[structopt(name = "disable-http-rpc", long)]
    ///disable http jsonrpc endpoint
    pub disable: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "http-apis", long)]
    ///rpc apiset to serve
    pub apis: Option<ApiSet>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "http-port", long)]
    /// Default http port is 9850
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "http-max-request-body",
        long,
        help = "max request body in bytes"
    )]
    ///max request body in bytes, Default is 10M
    pub max_request_body_size: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "http-threads", long)]
    /// How many thread to use for http service.
    pub threads: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "http-ip-headers", long, use_delimiter = true)]
    /// list of http header which identify a ip, Default: X-Real-IP,X-Forwarded-For
    pub ip_headers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "unsupported-rpc-protocols", long, use_delimiter = true)]
    unsupported_rpc_protocols: Option<Vec<String>>,
}

impl HttpConfiguration {
    pub fn max_request_body_size(&self) -> usize {
        self.max_request_body_size
            .unwrap_or(DEFAULT_MAX_REQUEST_BODY_SIZE)
    }
    pub fn threads(&self) -> usize {
        self.threads.unwrap_or_else(num_cpus::get)
    }
    pub fn apis(&self) -> &ApiSet {
        self.apis.as_ref().unwrap_or(&ApiSet::UnsafeContext)
    }
    pub fn ip_headers(&self) -> Vec<String> {
        self.ip_headers
            .clone()
            .unwrap_or_else(|| vec!["X-Real-IP".to_string(), "X-Forwarded-For".to_string()])
    }

    pub fn merge(&mut self, o: &Self) -> Result<()> {
        if o.disable {
            self.disable = true;
        }
        if o.apis.is_some() {
            self.apis = o.apis.clone();
        }
        if o.port.is_some() {
            self.port = o.port;
        }
        if o.max_request_body_size.is_some() {
            self.max_request_body_size = o.max_request_body_size;
        }
        if o.threads.is_some() {
            self.threads = o.threads;
        }
        if o.ip_headers.is_some() {
            let mut ip_headers: HashSet<String> = self
                .ip_headers
                .clone()
                .unwrap_or_default()
                .into_iter()
                .collect();
            ip_headers.extend(o.ip_headers.clone().unwrap_or_default());
            self.ip_headers = Some(ip_headers.into_iter().collect());
        }
        if o.unsupported_rpc_protocols.is_some() {
            let mut protocols: HashSet<String> = self
                .unsupported_rpc_protocols
                .clone()
                .unwrap_or_default()
                .into_iter()
                .collect();
            protocols.extend(o.unsupported_rpc_protocols.clone().unwrap_or_default());
            self.unsupported_rpc_protocols = Some(
                protocols
                    .into_iter()
                    .map(|protocol| protocol.to_lowercase())
                    .collect(),
            );
        }
        Ok(())
    }

    pub fn _unsupported_rpc_protocols(&self) -> Option<Vec<String>> {
        self.unsupported_rpc_protocols.clone()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct TcpConfiguration {
    #[serde(skip)]
    #[structopt(name = "disable-tcp-rpc", long, help = "disable tcp jsonrpc endpoint")]
    pub disable: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "tcp-apis", long, help = "rpc apiset to serve")]
    pub apis: Option<ApiSet>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "tcp-port", long)]
    /// Default tcp port is 9860
    pub port: Option<u16>,
}

impl TcpConfiguration {
    pub fn apis(&self) -> &ApiSet {
        self.apis.as_ref().unwrap_or(&ApiSet::UnsafeContext)
    }

    pub fn merge(&mut self, o: &Self) -> Result<()> {
        if o.disable {
            self.disable = true;
        }
        if o.apis.is_some() {
            self.apis = o.apis.clone();
        }
        if o.port.is_some() {
            self.port = o.port;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct WsConfiguration {
    #[serde(skip)]
    #[structopt(
        name = "disable-websocket-rpc",
        long,
        help = "disable websocket jsonrpc endpoint"
    )]
    pub disable: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "websocket-apis", long, help = "rpc apiset to serve")]
    pub apis: Option<ApiSet>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "websocket-port", long)]
    /// Default websocket port is 9870
    pub port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "websocket-max-request-body", long)]
    /// Max request body in bytes, Default is 10M
    pub max_request_body_size: Option<usize>,
}

impl WsConfiguration {
    pub fn max_request_body_size(&self) -> usize {
        self.max_request_body_size
            .unwrap_or(DEFAULT_MAX_REQUEST_BODY_SIZE)
    }
    pub fn apis(&self) -> &ApiSet {
        self.apis.as_ref().unwrap_or(&ApiSet::PubSub)
    }
    pub fn merge(&mut self, o: &Self) -> Result<()> {
        if o.disable {
            self.disable = true;
        }
        if o.apis.is_some() {
            self.apis = o.apis.clone();
        }
        if o.port.is_some() {
            self.port = o.port;
        }
        if o.max_request_body_size.is_some() {
            self.max_request_body_size = o.max_request_body_size;
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct IpcConfiguration {
    #[serde(skip)]
    #[structopt(name = "disable-ipc-rpc", long, help = "disable ipc jsonrpc endpoint")]
    pub disable: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "ipc-apis", long, help = "rpc apiset to serve")]
    pub apis: Option<ApiSet>,
}

impl IpcConfiguration {
    pub fn apis(&self) -> &ApiSet {
        self.apis.as_ref().unwrap_or(&ApiSet::IpcContext)
    }
    pub fn merge(&mut self, o: &Self) -> Result<()> {
        if o.disable {
            self.disable = true;
        }
        if o.apis.is_some() {
            self.apis = o.apis.clone();
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct ApiQuotaConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "jsonrpc-default-global-api-quota",
        long,
        help = "default api quota, eg: 1000/s"
    )]
    pub default_global_api_quota: Option<ApiQuotaConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
    name = "jsonrpc-custom-global-api-quota",
    long,
    help = "customize api quota, eg: node.info=100/s",
    number_of_values = 1,
    parse(try_from_str = parse_key_val)
    )]
    /// number_of_values = 1 forces the user to repeat the -D option for each key-value pair:
    /// my_program -D a=1 -D b=2
    pub custom_global_api_quota: Option<Vec<(String, ApiQuotaConfig)>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "jsonrpc-default-user-api-quota",
        long,
        help = "default api quota of user, eg: 1000/s"
    )]
    pub default_user_api_quota: Option<ApiQuotaConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
    name = "jsonrpc-custom-user-api-quota",
    long,
    help = "customize api quota of user, eg: node.info=100/s",
    number_of_values = 1,
    parse(try_from_str = parse_key_val)
    )]
    pub custom_user_api_quota: Option<Vec<(String, ApiQuotaConfig)>>,
}

impl ApiQuotaConfiguration {
    pub fn default_global_api_quota(&self) -> ApiQuotaConfig {
        self.default_global_api_quota
            .clone()
            .unwrap_or(ApiQuotaConfig {
                max_burst: NonZeroU32::new(1000).expect("New NonZeroU32 should success."),
                duration: QuotaDuration::Second,
            })
    }

    pub fn custom_global_api_quota(&self) -> Vec<(String, ApiQuotaConfig)> {
        self.custom_global_api_quota.clone().unwrap_or_default()
    }

    pub fn default_user_api_quota(&self) -> ApiQuotaConfig {
        self.default_user_api_quota
            .clone()
            .unwrap_or(ApiQuotaConfig {
                max_burst: NonZeroU32::new(50).expect("New NonZeroU32 should success."),
                duration: QuotaDuration::Second,
            })
    }

    pub fn custom_user_api_quota(&self) -> Vec<(String, ApiQuotaConfig)> {
        self.custom_user_api_quota.clone().unwrap_or_default()
    }

    pub fn merge(&mut self, o: &Self) -> Result<()> {
        if o.default_global_api_quota.is_some() {
            self.default_global_api_quota = o.default_global_api_quota.clone();
        }
        //TODO should merge two vec?
        if o.custom_global_api_quota.is_some() {
            self.custom_global_api_quota = o.custom_global_api_quota.clone();
        }
        if o.default_user_api_quota.is_some() {
            self.default_user_api_quota = o.default_user_api_quota.clone();
        }
        if o.custom_user_api_quota.is_some() {
            self.custom_user_api_quota = o.custom_user_api_quota.clone();
        }
        Ok(())
    }
}

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct RpcConfig {
    #[serde(default)]
    #[structopt(flatten)]
    pub http: HttpConfiguration,

    #[serde(default)]
    #[structopt(flatten)]
    pub tcp: TcpConfiguration,

    #[serde(default)]
    #[structopt(flatten)]
    pub ws: WsConfiguration,

    #[serde(default)]
    #[structopt(flatten)]
    pub ipc: IpcConfiguration,

    #[serde(default)]
    #[structopt(flatten)]
    pub api_quotas: ApiQuotaConfiguration,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long = "rpc-address")]
    /// Rpc address, default is 0.0.0.0
    pub rpc_address: Option<IpAddr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long = "event-query-max-block-range")]
    pub block_query_max_range: Option<u64>,

    #[serde(skip)]
    #[structopt(skip)]
    http_address: Option<ListenAddress>,

    #[serde(skip)]
    #[structopt(skip)]
    tcp_address: Option<ListenAddress>,

    #[serde(skip)]
    #[structopt(skip)]
    ws_address: Option<ListenAddress>,

    #[serde(skip)]
    #[structopt(skip)]
    base: Option<Arc<BaseConfig>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long = "query-max-txn-info-range")]
    pub txn_info_query_max_range: Option<u64>,
}

#[derive(Clone, Eq, PartialEq)]
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

#[allow(clippy::from_over_into)]
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
    pub fn rpc_address(&self) -> IpAddr {
        self.rpc_address.unwrap_or(DEFAULT_RPC_ADDRESS)
    }

    pub fn get_ipc_file(&self) -> PathBuf {
        let base = self.base();
        Self::get_ipc_file_by_base(base)
    }

    pub fn get_http_address(&self) -> Option<ListenAddress> {
        self.http_address.clone()
    }

    pub fn get_tcp_address(&self) -> Option<ListenAddress> {
        self.tcp_address.clone()
    }

    pub fn get_ws_address(&self) -> Option<ListenAddress> {
        self.ws_address.clone()
    }

    pub fn block_query_max_range(&self) -> u64 {
        self.block_query_max_range
            .unwrap_or(DEFAULT_BLOCK_QUERY_MAX_RANGE)
    }

    pub fn txn_info_query_max_range(&self) -> u64 {
        self.txn_info_query_max_range
            .unwrap_or(DEFAULT_TXN_INFO_QUEYR_MAX_RANGE)
    }

    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    fn generate_address(&mut self) {
        let base = self.base();
        let (http_port, tcp_port, ws_port) = if base.net().is_test() {
            let ports = get_random_available_ports(3);
            (
                self.http.port.unwrap_or(ports[0]),
                self.tcp.port.unwrap_or(ports[1]),
                self.ws.port.unwrap_or(ports[2]),
            )
        } else if base.net().is_dev() {
            (
                self.http
                    .port
                    .unwrap_or_else(|| get_available_port_from(DEFAULT_HTTP_PORT)),
                self.tcp
                    .port
                    .unwrap_or_else(|| get_available_port_from(DEFAULT_TCP_PORT)),
                self.ws
                    .port
                    .unwrap_or_else(|| get_available_port_from(DEFAULT_WEB_SOCKET_PORT)),
            )
        } else {
            (
                self.http.port.unwrap_or(DEFAULT_HTTP_PORT),
                self.tcp.port.unwrap_or(DEFAULT_TCP_PORT),
                self.ws.port.unwrap_or(DEFAULT_WEB_SOCKET_PORT),
            )
        };
        self.http_address = if self.http.disable {
            None
        } else {
            Some(ListenAddress::new("http", self.rpc_address(), http_port))
        };
        self.tcp_address = if self.tcp.disable {
            None
        } else {
            Some(ListenAddress::new("tcp", self.rpc_address(), tcp_port))
        };
        self.ws_address = if self.ws.disable {
            None
        } else {
            Some(ListenAddress::new("ws", self.rpc_address(), ws_port))
        };
    }

    #[cfg(not(windows))]
    fn get_ipc_file_by_base(base: &BaseConfig) -> PathBuf {
        base.data_dir().join(DEFAULT_IPC_FILE)
    }

    #[cfg(windows)]
    fn get_ipc_file_by_base(base: &BaseConfig) -> PathBuf {
        PathBuf::from(r"\\.\pipe")
            .join("starcoin")
            .join(base.net().id().dir_name())
            .join(DEFAULT_IPC_FILE)
    }
}

impl ConfigModule for RpcConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);
        if opt.rpc.rpc_address.is_some() {
            self.rpc_address = opt.rpc.rpc_address;
        }
        if opt.rpc.block_query_max_range.is_some() {
            self.block_query_max_range = opt.rpc.block_query_max_range;
        }
        if opt.rpc.txn_info_query_max_range.is_some() {
            self.txn_info_query_max_range = opt.rpc.txn_info_query_max_range;
        }
        self.http.merge(&opt.rpc.http)?;
        self.tcp.merge(&opt.rpc.tcp)?;
        self.ws.merge(&opt.rpc.ws)?;
        self.ipc.merge(&opt.rpc.ipc)?;
        self.api_quotas.merge(&opt.rpc.api_quotas)?;

        self.generate_address();

        info!("Http rpc address: {:?}", self.get_http_address());
        info!("TCP rpc address: {:?}", self.get_tcp_address());
        info!("Websocket rpc address: {:?}", self.get_ws_address());
        info!("Ipc file path: {:?}", self.get_ipc_file());

        Ok(())
    }
}
