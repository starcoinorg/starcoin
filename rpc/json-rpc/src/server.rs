// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::NodeConfig;
use jsonrpc_core::IoHandler;
use jsonrpc_http_server;
use jsonrpc_server_utils::cors::AccessControlAllowOrigin;
use jsonrpc_server_utils::hosts::DomainsValidation;
use jsonrpc_tcp_server;
use jsonrpc_ws_server;
use std::sync::Arc;

pub struct RpcServer {
    http: jsonrpc_http_server::Server,
    tcp: Option<jsonrpc_tcp_server::Server>,
    ws: Option<jsonrpc_ws_server::Server>,
}

impl RpcServer {
    pub fn new(config: Arc<NodeConfig>, mut io_handler: IoHandler) -> RpcServer {
        io_handler.add_method("status", |_| jsonrpc_core::futures::future::ok("ok".into()));

        let http = jsonrpc_http_server::ServerBuilder::new(io_handler.clone())
            .cors(DomainsValidation::AllowOnly(vec![
                AccessControlAllowOrigin::Null,
                AccessControlAllowOrigin::Any,
            ]))
            .threads(config.rpc.threads.unwrap_or_else(num_cpus::get))
            .max_request_body_size(config.rpc.max_request_body_size)
            .health_api(("/status", "status"))
            .start_http(&config.rpc.http_address)
            .expect("Unable to start RPC server");
        println!("Json-rpc start at :{}", &config.rpc.http_address);
        RpcServer {
            http,
            tcp: None,
            ws: None,
        }
    }

    pub fn close(self) {
        self.http.close();
        if let Some(tcp) = self.tcp {
            tcp.close();
        }
        if let Some(ws) = self.ws {
            ws.close();
        }
    }
}
