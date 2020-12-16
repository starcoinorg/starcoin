// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpc_http_server::hyper;
use jsonrpc_pubsub::Session;
use starcoin_rpc_api::metadata::Metadata;
use std::net::IpAddr;
use std::sync::Arc;

/// Common HTTP & IPC & TCP metadata extractor.
#[derive(Default)]
pub struct RpcExtractor {
    pub http_ip_headers: Vec<String>,
}

impl jsonrpc_http_server::MetaExtractor<Metadata> for RpcExtractor {
    fn read_metadata(&self, _req: &hyper::Request<hyper::Body>) -> Metadata {
        let mut client_ip = None;
        for header in self.http_ip_headers.iter() {
            if let Some(v) = _req.headers().get(header) {
                if let Ok(s) = v.to_str() {
                    // if it's an valid ip.
                    if let Some(Ok(ip)) = s.split(',').next().map(|s| s.trim().parse::<IpAddr>()) {
                        client_ip = Some(ip);
                    }
                }
            }
        }

        Metadata {
            session: None,
            user: client_ip.map(|ip| ip.to_string()),
        }
    }
}
impl jsonrpc_ipc_server::MetaExtractor<Metadata> for RpcExtractor {
    fn extract(&self, req: &jsonrpc_ipc_server::RequestContext) -> Metadata {
        Metadata {
            session: Some(Arc::new(Session::new(req.sender.clone()))),
            user: None,
        }
    }
}
impl jsonrpc_tcp_server::MetaExtractor<Metadata> for RpcExtractor {
    fn extract(&self, context: &jsonrpc_tcp_server::RequestContext) -> Metadata {
        Metadata {
            session: Some(Arc::new(Session::new(context.sender.clone()))),
            user: Some(context.peer_addr.ip().to_string()),
        }
    }
}

pub struct WsExtractor;
impl jsonrpc_ws_server::MetaExtractor<Metadata> for WsExtractor {
    fn extract(&self, req: &jsonrpc_ws_server::RequestContext) -> Metadata {
        let session = Some(Arc::new(Session::new(req.sender())));
        Metadata {
            session,
            user: None,
        }
    }
}
