// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::module::{StatusRpcImpl, TxPoolRpcImpl};
use crate::server::RpcServer;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use jsonrpc_core::IoHandler;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::{status::StatusApi, txpool::TxPoolApi};
use std::cell::RefCell;
use std::sync::Arc;
use traits::TxPoolAsyncService;

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
        Self::launch_with_apis(
            config,
            Some(StatusRpcImpl::new()),
            Some(TxPoolRpcImpl::new(txpool_service)),
        )
    }

    pub fn launch_with_apis<S, T>(
        config: Arc<NodeConfig>,
        status_api: Option<S>,
        txpool_api: Option<T>,
    ) -> Result<Addr<Self>>
    where
        S: StatusApi,
        T: TxPoolApi,
    {
        let mut io_handler = IoHandler::new();
        if let Some(status_api) = status_api {
            io_handler.extend_with(StatusApi::to_delegate(status_api));
        }
        if let Some(txpool_api) = txpool_api {
            io_handler.extend_with(TxPoolApi::to_delegate(txpool_api));
        }
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

    pub fn attach(&self) {}
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
    use jsonrpc_core_client::transports::{http, local};
    use traits::mock::MockTxPoolService;

    #[stest::test]
    async fn test_start() {
        let config = Arc::new(NodeConfig::random_for_test());
        let txpool = MockTxPoolService::new();
        let _rpc_actor = JSONRpcActor::launch(config, txpool).unwrap();
    }
}
