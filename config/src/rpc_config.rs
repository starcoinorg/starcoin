// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::get_available_port;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024; //10M

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RpcConfig {
    ipc_file: PathBuf,
    /// The address for http rpc.
    pub http_address: Option<SocketAddr>,
    /// The address for tcp rpc notification.
    pub tcp_address: Option<SocketAddr>,
    /// The address for websocket rpc notification.
    pub ws_address: Option<SocketAddr>,
    pub max_request_body_size: usize,
    pub threads: Option<usize>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            ipc_file: Path::new("starcoin.ipc").to_path_buf(),
            //TODO http,ws,tcp should be disabled at default.
            http_address: Some("127.0.0.1:9830".parse::<SocketAddr>().unwrap()),
            ws_address: Some("127.0.0.1:9831".parse::<SocketAddr>().unwrap()),
            tcp_address: Some("127.0.0.1:9832".parse::<SocketAddr>().unwrap()),
            max_request_body_size: DEFAULT_MAX_REQUEST_BODY_SIZE,
            threads: None,
        }
    }
}

impl RpcConfig {
    pub fn random_for_test() -> Self {
        let mut config = Self::default();
        config.http_address = Some(
            format!("127.0.0.1:{}", get_available_port())
                .parse::<SocketAddr>()
                .unwrap(),
        );
        config.tcp_address = Some(
            format!("127.0.0.1:{}", get_available_port())
                .parse::<SocketAddr>()
                .unwrap(),
        );
        config.ws_address = Some(
            format!("127.0.0.1:{}", get_available_port())
                .parse::<SocketAddr>()
                .unwrap(),
        );
        config
    }

    pub fn get_ipc_file<P: AsRef<Path>>(&self, data_dir: P) -> PathBuf {
        data_dir.as_ref().join(self.ipc_file.as_path())
    }
}
