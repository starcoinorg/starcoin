// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::module::{AccountRpcImpl, StatusRpcImpl, TxPoolRpcImpl};
use crate::server::RpcServer;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use jsonrpc_core::IoHandler;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::account::AccountApi;
use starcoin_rpc_api::{status::StatusApi, txpool::TxPoolApi};
use starcoin_wallet_api::WalletAsyncService;
use std::cell::RefCell;
use std::sync::Arc;
use traits::TxPoolAsyncService;

pub mod module;
pub mod server;

pub struct JSONRpcActor {
    config: Arc<NodeConfig>,
    io_handler: IoHandler,
    server: RefCell<Option<RpcServer>>,
}

impl JSONRpcActor {
    pub fn launch<TS, AS>(
        config: Arc<NodeConfig>,
        txpool_service: TS,
        account_service: AS,
    ) -> Result<(Addr<JSONRpcActor>, IoHandler)>
    where
        TS: TxPoolAsyncService + 'static,
        AS: WalletAsyncService + 'static,
    {
        Self::launch_with_apis(
            config,
            Some(TxPoolRpcImpl::new(txpool_service)),
            Some(AccountRpcImpl::new(account_service)),
        )
    }

    pub fn launch_with_apis<T, A>(
        config: Arc<NodeConfig>,
        txpool_api: Option<T>,
        account_api: Option<A>,
    ) -> Result<(Addr<Self>, IoHandler)>
    where
        T: TxPoolApi,
        A: AccountApi,
    {
        let mut io_handler = IoHandler::new();
        io_handler.extend_with(StatusApi::to_delegate(StatusRpcImpl::new()));
        if let Some(txpool_api) = txpool_api {
            io_handler.extend_with(TxPoolApi::to_delegate(txpool_api));
        }
        if let Some(account_api) = account_api {
            io_handler.extend_with(AccountApi::to_delegate(account_api));
        }
        Self::launch_with_handler(config, io_handler)
    }

    pub fn launch_with_handler(
        config: Arc<NodeConfig>,
        io_handler: IoHandler,
    ) -> Result<(Addr<Self>, IoHandler)> {
        let actor = JSONRpcActor {
            config,
            server: RefCell::new(None),
            io_handler: io_handler.clone(),
        };
        Ok((actor.start(), io_handler))
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
    use starcoin_wallet_api::mock::MockWalletService;
    use traits::mock::MockTxPoolService;

    #[stest::test]
    async fn test_start() {
        let config = Arc::new(NodeConfig::random_for_test());
        let txpool = MockTxPoolService::new();
        let account_service = MockWalletService::new().unwrap();
        let _rpc_actor = JSONRpcActor::launch(config, txpool, account_service).unwrap();
    }
}
