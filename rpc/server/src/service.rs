// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpc_core::IoHandler;
use jsonrpc_http_server;
use jsonrpc_server_utils::cors::AccessControlAllowOrigin;
use jsonrpc_server_utils::hosts::DomainsValidation;
use jsonrpc_tcp_server;
use jsonrpc_ws_server;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use std::sync::Arc;

pub struct RpcService {
    ipc: jsonrpc_ipc_server::Server,
    http: Option<jsonrpc_http_server::Server>,
    tcp: Option<jsonrpc_tcp_server::Server>,
    ws: Option<jsonrpc_ws_server::Server>,
}

impl RpcService {
    pub fn new(config: Arc<NodeConfig>, io_handler: IoHandler) -> RpcService {
        let ipc_file = config.rpc.get_ipc_file();
        let ipc = jsonrpc_ipc_server::ServerBuilder::new(io_handler.clone())
            .start(ipc_file.to_str().expect("Path to string should success."))
            .expect(format!("Unable to start IPC server with ipc file: {:?}", ipc_file).as_str());
        info!("Ipc rpc server start at :{:?}", ipc_file);
        let http = match &config.rpc.http_address {
            Some(address) => {
                let http = jsonrpc_http_server::ServerBuilder::new(io_handler)
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

        RpcService {
            ipc,
            http,
            tcp: None,
            ws: None,
        }
    }

    pub fn close(self) {
        self.ipc.close();
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
