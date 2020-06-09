// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_available_port, get_available_port_multi, BaseConfig, ChainNetwork, ConfigModule,
    StarcoinOpt,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;
//10M
const DEFAULT_IPC_FILE: &str = "starcoin.ipc";
const DEFAULT_HTTP_PORT: u16 = 9850;
const DEFAULT_TCP_PORT: u16 = 9851;
const DEFAULT_WEB_SOCKET_PORT: u16 = 9852;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
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

impl Default for RpcConfig {
    fn default() -> Self {
        Self::default_with_net(ChainNetwork::default())
    }
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
    fn default_with_net(net: ChainNetwork) -> Self {
        let port = match net {
            ChainNetwork::Dev => get_available_port(),
            _ => DEFAULT_HTTP_PORT,
        };
        let http_address = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();
        let tcp_address = {
            let port = match net {
                ChainNetwork::Dev => get_available_port(),
                _ => DEFAULT_TCP_PORT,
            };
            format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap()
        };
        let ws_address = {
            let port = match net {
                ChainNetwork::Dev => get_available_port(),
                _ => DEFAULT_WEB_SOCKET_PORT,
            };
            format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap()
        };
        Self {
            http_address: Some(http_address),
            ws_address: Some(ws_address),
            tcp_address: Some(tcp_address),
            max_request_body_size: DEFAULT_MAX_REQUEST_BODY_SIZE,
            threads: None,
            ipc_file_path: None,
        }
    }

    fn random(&mut self, base: &BaseConfig) {
        let ports = get_available_port_multi(3);
        self.http_address = Some(
            format!("127.0.0.1:{}", ports[0])
                .parse::<SocketAddr>()
                .unwrap(),
        );
        self.tcp_address = Some(
            format!("127.0.0.1:{}", ports[1])
                .parse::<SocketAddr>()
                .unwrap(),
        );
        self.ws_address = Some(
            format!("127.0.0.1:{}", ports[2])
                .parse::<SocketAddr>()
                .unwrap(),
        );
        self.ipc_file_path = Some(Self::get_ipc_file_by_base(base))
    }

    fn load(&mut self, base: &BaseConfig, opt: &StarcoinOpt) -> Result<()> {
        let ipc_file_path = Self::get_ipc_file_by_base(base);
        info!("Ipc file path: {:?}", ipc_file_path);
        info!("Http rpc address: {}", self.http_address.unwrap());
        self.ipc_file_path = Some(ipc_file_path);
        if let Some(rpc_address) = &opt.rpc_address {
            self.ws_address =
                Some(format!("{}:{}", rpc_address, DEFAULT_WEB_SOCKET_PORT).parse::<SocketAddr>()?);
            self.http_address =
                Some(format!("{}:{}", rpc_address, DEFAULT_HTTP_PORT).parse::<SocketAddr>()?);
            self.tcp_address =
                Some(format!("{}:{}", rpc_address, DEFAULT_TCP_PORT).parse::<SocketAddr>()?);
        }
        Ok(())
    }
}
