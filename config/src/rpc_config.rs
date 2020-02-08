// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

const DEFAULT_MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024; //10M

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RpcConfig {
    /// The address for http rpc.
    pub http_address: SocketAddr,
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
            http_address: "0.0.0.0:9830".parse::<SocketAddr>().unwrap(),
            ws_address: Some("0.0.0.0:9831".parse::<SocketAddr>().unwrap()),
            tcp_address: Some("0.0.0.0:9832".parse::<SocketAddr>().unwrap()),
            max_request_body_size: DEFAULT_MAX_REQUEST_BODY_SIZE,
            threads: None,
        }
    }
}
