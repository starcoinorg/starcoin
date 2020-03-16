// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.

use crate::module::{TxPoolRpc, TxPoolRpcImpl};
use crate::server::RpcServer;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use jsonrpc_core::IoHandler;
use starcoin_logger::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;
use traits::{mock::MockTxPoolService, TxPoolAsyncService};

mod module;
mod server;

pub struct JSONRpcActor {
    config: Arc<NodeConfig>,
    io_handler: IoHandler,
    server: RefCell<Option<RpcServer>>,
}

impl JSONRpcActor {
    pub fn launch<TS>(config: Arc<NodeConfig>, txpool_service: TS) -> Result<Addr<JSONRpcActor>>
    where
        TS: TxPoolAsyncService + 'static,
    {
        let mut io_handler = IoHandler::new();
        io_handler.extend_with(TxPoolRpcImpl::new(txpool_service).to_delegate());
        Ok(JSONRpcActor {
            config,
            server: RefCell::new(None),
            io_handler,
        }
        .start())
    }

    fn do_start(&mut self) {
        let server = RpcServer::new(self.config.clone(), self.io_handler.clone());
        self.server.replace(Some(server));
    }

    fn do_stop(&mut self) {
        let server = self.server.replace(None);
        match server {
            Some(server) => server.close(),
            None => {}
        }
    }
}

impl Actor for JSONRpcActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.do_start();
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.do_stop();
        Running::Stop
    }
}

impl Supervised for JSONRpcActor {
    fn restarting(&mut self, _ctx: &mut Self::Context) {
        info!("Restart JSON rpc service.");
        self.do_stop();
        self.do_start();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[stest::test]
    async fn test_start() {
        let config = Arc::new(NodeConfig::random_for_test());
        let txpool = MockTxPoolService::new();
        let _rpc_actor = JSONRpcActor::launch(config, txpool).unwrap();
    }
}
