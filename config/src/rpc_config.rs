// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port_from, get_random_available_ports, BaseConfig, ChainNetwork, ConfigModule,
    StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

const DEFAULT_MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;
//10M
const DEFAULT_IPC_FILE: &str = "starcoin.ipc";
const DEFAULT_HTTP_PORT: u16 = 9850;
const DEFAULT_TCP_PORT: u16 = 9860;
const DEFAULT_WEB_SOCKET_PORT: u16 = 9870;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcConfig {
    /// The address for http rpc.
    pub http_address: Option<SocketAddr>,
    /// The address for tcp rpc notification.
    pub tcp_address: Option<SocketAddr>,
    /// The address for websocket rpc notification.
    pub ws_address: Option<SocketAddr>,
    pub max_request_body_size: usize,
    pub threads: Option<usize>,
    #[serde(skip)]
    ipc_file_path: Option<PathBuf>,
}

impl RpcConfig {
    pub fn get_ipc_file(&self) -> &Path {
        self.ipc_file_path
            .as_ref()
            .expect("config should init first.")
    }

    pub fn get_http_address(&self) -> Option<String> {
        self.http_address
            .as_ref()
            .map(|addr| format!("http://{}", addr))
    }

    pub fn get_ipc_file_by_base(base: &BaseConfig) -> PathBuf {
        base.data_dir().join(DEFAULT_IPC_FILE)
    }
}

impl ConfigModule for RpcConfig {
    fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self> {
        let ports = match base.net {
            ChainNetwork::Test => get_random_available_ports(3),
            ChainNetwork::Dev => vec![
                get_available_port_from(DEFAULT_HTTP_PORT),
                get_available_port_from(DEFAULT_TCP_PORT),
                get_available_port_from(DEFAULT_WEB_SOCKET_PORT),
            ],
            _ => vec![DEFAULT_HTTP_PORT, DEFAULT_TCP_PORT, DEFAULT_WEB_SOCKET_PORT],
        };
        let rpc_address: IpAddr = opt
            .rpc_address
            .clone()
            .unwrap_or_else(|| "127.0.0.1".to_string())
            .parse()?;

        let http_address = Some(
            format!("{}:{}", rpc_address, ports[0])
                .parse::<SocketAddr>()
                .unwrap(),
        );

        let tcp_address = Some(
            format!("{}:{}", rpc_address, ports[1])
                .parse::<SocketAddr>()
                .unwrap(),
        );

        let ws_address = Some(
            format!("{}:{}", rpc_address, ports[2])
                .parse::<SocketAddr>()
                .unwrap(),
        );

        Ok(Self {
            http_address,
            ws_address,
            tcp_address,
            max_request_body_size: DEFAULT_MAX_REQUEST_BODY_SIZE,
            threads: None,
            ipc_file_path: None,
        })
    }

    fn after_load(&mut self, _opt: &StarcoinOpt, base: &BaseConfig) -> Result<()> {
        self.ipc_file_path = Some(Self::get_ipc_file_by_base(base));
        info!("Ipc file path: {:?}", self.ipc_file_path);
        info!("Http rpc address: {:?}", self.http_address);
        info!("TCP rpc address: {:?}", self.tcp_address);
        info!("Websocket rpc address: {:?}", self.ws_address);
        Ok(())
    }
}
