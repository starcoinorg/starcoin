// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.

use crate::module::{TxPoolRpc, TxPoolRpcImpl};
use crate::server::RpcServer;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use jsonrpc_core::IoHandler;
use std::sync::Arc;
use txpool::TxPoolRef;

mod module;
mod server;

pub struct JSONRpcActor {
    _server: RpcServer,
}

impl JSONRpcActor {
    pub fn launch(config: Arc<NodeConfig>, txpool_ref: TxPoolRef) -> Result<Addr<JSONRpcActor>> {
        let mut io_handler = IoHandler::new();
        io_handler.extend_with(TxPoolRpcImpl::new(txpool_ref).to_delegate());
        let server = RpcServer::new(config, io_handler);
        Ok(JSONRpcActor { _server: server }.start())
    }
}

impl Actor for JSONRpcActor {
    type Context = Context<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        //TODO stop
        //self.server.close();
        Running::Stop
    }
}

impl Supervised for JSONRpcActor {
    fn restarting(&mut self, _ctx: &mut Self::Context) {
        //TODO
        println!("Restart JSON rpc service started.");
    }
}
