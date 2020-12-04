// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_ports, ApiSet, BaseConfig, ConfigModule,
    StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use std::net::IpAddr;
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
    #[structopt(name = "http-port", long, default_value = "9850")]
    pub port: u16,
    #[structopt(
        name = "http-max-request-body",
        long,
        help = "max request body in bytes",
        default_value = "10485760"
    )]
    pub max_request_body_size: usize,
    #[structopt(name = "http-threads", long, help = "threads to use")]
    pub threads: Option<usize>,
}

impl Default for HttpConfiguration {
    fn default() -> Self {
        Self {
            disable: false,
            apis: ApiSet::UnsafeContext,
            max_request_body_size: DEFAULT_MAX_REQUEST_BODY_SIZE,
            threads: None,
            port: DEFAULT_HTTP_PORT,
        }
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
    #[structopt(name = "tcp-port", long, default_value = "9860")]
    pub port: u16,
}

impl Default for TcpConfiguration {
    fn default() -> Self {
        Self {
            disable: false,
            apis: ApiSet::UnsafeContext,
            port: DEFAULT_TCP_PORT,
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
    #[structopt(name = "websocket-port", long, default_value = "9870")]
    pub port: u16,
    #[structopt(
        name = "websocket-max-request-body",
        long,
        help = "max request body in bytes",
        default_value = "10485760"
    )]
    pub max_request_body_size: usize,
}

impl Default for WsConfiguration {
    fn default() -> Self {
        Self {
            disable: false,
            apis: ApiSet::PubSub,
            max_request_body_size: DEFAULT_MAX_REQUEST_BODY_SIZE,
            port: DEFAULT_WEB_SOCKET_PORT,
        }
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcConfig {
    pub tcp: TcpConfiguration,
    pub http: HttpConfiguration,
    pub ws: WsConfiguration,
    pub ipc: IpcConfiguration,
    pub rpc_address: IpAddr,
}

impl RpcConfig {
    pub fn get_ipc_file(&self) -> &Path {
        self.ipc
            .ipc_file_path
            .as_ref()
            .expect("config should init first.")
    }

    pub fn get_http_address(&self) -> Option<String> {
        if self.http.disable {
            return None;
        }
        Some(format!("http://{}:{}", self.rpc_address, self.http.port))
    }
    pub fn get_tcp_address(&self) -> Option<String> {
        if self.tcp.disable {
            return None;
        }
        Some(format!("tcp://{}:{}", self.rpc_address, self.tcp.port))
    }

    pub fn get_ws_address(&self) -> Option<String> {
        if self.ws.disable {
            return None;
        }
        Some(format!("ws://{}:{}", self.rpc_address, self.ws.port))
    }
    pub fn get_ipc_file_by_base(base: &BaseConfig) -> PathBuf {
        base.data_dir().join(DEFAULT_IPC_FILE)
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
            rpc_address,
        };

        if base.net.is_test() {
            let ports = get_random_available_ports(3);
            config.http.port = ports[0];
            config.tcp.port = ports[1];
            config.ws.port = ports[2];
        } else if base.net.is_dev() {
            config.http.port = get_available_port_from(DEFAULT_HTTP_PORT);

            config.tcp.port = get_available_port_from(DEFAULT_TCP_PORT);
            config.ws.port = get_available_port_from(DEFAULT_WEB_SOCKET_PORT);
        }

        if config.ipc.ipc_file_path.is_none() {
            config.ipc.ipc_file_path = Some(Self::get_ipc_file_by_base(base));
        }

        Ok(config)
    }

    fn after_load(&mut self, _opt: &StarcoinOpt, _base: &BaseConfig) -> Result<()> {
        info!("Ipc file path: {:?}", self.ipc.ipc_file_path);
        info!("Http rpc address: {:?}", self.get_http_address());
        info!("TCP rpc address: {:?}", self.get_tcp_address());
        info!("Websocket rpc address: {:?}", self.get_ws_address());
        Ok(())
    }
}
