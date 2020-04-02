// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{get_available_port_multi, BaseConfig, ChainNetwork, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024; //10M

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RpcConfig {
    /// The ipc file name.
    ipc_file: PathBuf,
    /// The address for http rpc.
    pub http_address: Option<SocketAddr>,
    /// The address for tcp rpc notification.
    pub tcp_address: Option<SocketAddr>,
    /// The address for websocket rpc notification.
    pub ws_address: Option<SocketAddr>,
    pub max_request_body_size: usize,
    pub threads: Option<usize>,
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
}

impl ConfigModule for RpcConfig {
    fn default_with_net(_net: ChainNetwork) -> Self {
        Self {
            ipc_file: "starcoin.ipc".into(),
            http_address: None,
            ws_address: None,
            tcp_address: None,
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
        self.ipc_file_path = Some(base.data_dir().join(self.ipc_file.as_path()))
    }

    fn load(&mut self, base: &BaseConfig, _opt: &StarcoinOpt) -> Result<()> {
        self.ipc_file_path = Some(base.data_dir().join(self.ipc_file.as_path()));
        Ok(())
    }
}
