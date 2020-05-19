// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::extractors::{RpcExtractor, WsExtractor};
use crate::metadata::Metadata;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_server_utils::cors::AccessControlAllowOrigin;
use jsonrpc_server_utils::hosts::DomainsValidation;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_rpc_middleware::MetricMiddleware;
use std::sync::Arc;

pub struct RpcService {
    ipc: Option<jsonrpc_ipc_server::Server>,
    http: Option<jsonrpc_http_server::Server>,
    tcp: Option<jsonrpc_tcp_server::Server>,
    ws: Option<jsonrpc_ws_server::Server>,
}

impl RpcService {
    pub fn new(
        config: Arc<NodeConfig>,
        io_handler: MetaIoHandler<Metadata, MetricMiddleware>,
    ) -> RpcService {
        let ipc = Self::start_ipc(&config, io_handler.clone());
        let http = match &config.rpc.http_address {
            Some(address) => {
                let http = jsonrpc_http_server::ServerBuilder::new(io_handler.clone())
                    .meta_extractor(RpcExtractor)
                    .cors(DomainsValidation::AllowOnly(vec![
                        AccessControlAllowOrigin::Null,
                        AccessControlAllowOrigin::Any,
                    ]))
                    .threads(config.rpc.threads.unwrap_or_else(num_cpus::get))
                    .max_request_body_size(config.rpc.max_request_body_size)
                    .health_api(("/status", "status"))
                    .start_http(address)
                    .expect("Unable to start RPC server.");
                info!("Http rpc server start at :{}", address);
                Some(http)
            }
            None => None,
        };
        let tcp_server = match &config.rpc.tcp_address {
            Some(address) => {
                let tcp_server = jsonrpc_tcp_server::ServerBuilder::new(io_handler.clone())
                    .session_meta_extractor(RpcExtractor)
                    .start(address)
                    .expect("rpc: start tcp server should ok");
                info!("Rpc: tcp server start at: {}", address);
                Some(tcp_server)
            }
            None => None,
        };

        let ws_server = match &config.rpc.ws_address {
            None => None,
            Some(address) => {
                let ws_server = jsonrpc_ws_server::ServerBuilder::new(io_handler)
                    .session_meta_extractor(WsExtractor)
                    .max_payload(config.rpc.max_request_body_size)
                    .start(address)
                    .expect("rpc: start ws server should ok");
                info!("Rpc: websocket server start at: {}", address);
                Some(ws_server)
            }
        };

        RpcService {
            ipc,
            http,
            tcp: tcp_server,
            ws: ws_server,
        }
    }

    #[cfg(not(windows))]
    fn start_ipc(
        config: &NodeConfig,
        io_handler: MetaIoHandler<Metadata, MetricMiddleware>,
    ) -> Option<jsonrpc_ipc_server::Server> {
        let ipc_file = config.rpc.get_ipc_file();
        info!("Ipc rpc server start at :{:?}", ipc_file);
        Some(
            jsonrpc_ipc_server::ServerBuilder::new(io_handler)
                .session_meta_extractor(RpcExtractor)
                .start(ipc_file.to_str().expect("Path to string should success."))
                .unwrap_or_else(|e| {
                    panic!(
                        "Unable to start IPC server with ipc file: {:?}, error: {:?}",
                        ipc_file, e
                    )
                }),
        )
    }

    //IPC raise a error on windows: The filename, directory name, or volume label syntax is incorrect.
    #[cfg(windows)]
    fn start_ipc(
        _config: &NodeConfig,
        io_handler: MetaIoHandler<Metadata, MetricMiddleware>,
    ) -> Option<jsonrpc_ipc_server::Server> {
        None
    }

    pub fn close(self) {
        if let Some(ipc) = self.ipc {
            ipc.close();
        }
        if let Some(http) = self.http {
            http.close();
        }
        if let Some(tcp) = self.tcp {
            tcp.close();
        }
        if let Some(ws) = self.ws {
            ws.close();
        }
        info!("Rpc Sever is closed.");
    }
}
