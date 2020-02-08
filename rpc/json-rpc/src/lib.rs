// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.

use crate::module::{TxPoolRpc, TxPoolRpcImpl};
use crate::server::RpcServer;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use jsonrpc_core::IoHandler;
use txpool::TxPoolActor;

mod module;
mod server;

pub struct JSONRpcActor {
    server: RpcServer,
}

impl JSONRpcActor {
    pub fn launch(
        config: &NodeConfig,
        txpool_actor_ref: Addr<TxPoolActor>,
    ) -> Result<Addr<JSONRpcActor>> {
        let mut io_handler = IoHandler::new();
        io_handler.extend_with(TxPoolRpcImpl::new(txpool_actor_ref).to_delegate());
        let server = RpcServer::new(config, io_handler);
        Ok(JSONRpcActor { server }.start())
    }
}

impl Actor for JSONRpcActor {
    type Context = Context<Self>;
}

impl Supervised for JSONRpcActor {
    fn restarting(&mut self, _ctx: &mut Self::Context) {
        //TODO
        println!("Restart JSON rpc service started.");
    }
}
