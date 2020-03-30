// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::module::{AccountRpcImpl, NodeRpcImpl, StateRpcImpl, TxPoolRpcImpl};
use crate::server::RpcServer;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use jsonrpc_core::IoHandler;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::account::AccountApi;
use starcoin_rpc_api::{node::NodeApi, state::StateApi, txpool::TxPoolApi};
use starcoin_state_api::ChainStateAsyncService;
use starcoin_traits::TxPoolAsyncService;
use starcoin_wallet_api::WalletAsyncService;
use std::cell::RefCell;
use std::sync::Arc;

pub mod module;
pub mod server;

pub struct JSONRpcActor {
    config: Arc<NodeConfig>,
    io_handler: IoHandler,
    server: RefCell<Option<RpcServer>>,
}

impl JSONRpcActor {
    pub fn launch<TS, AS, SS>(
        config: Arc<NodeConfig>,
        txpool_service: TS,
        account_service: AS,
        state_service: SS,
    ) -> Result<(Addr<JSONRpcActor>, IoHandler)>
    where
        TS: TxPoolAsyncService + 'static,
        AS: WalletAsyncService + 'static,
        SS: ChainStateAsyncService + 'static,
    {
        Self::launch_with_apis(
            config,
            Some(TxPoolRpcImpl::new(txpool_service)),
            Some(AccountRpcImpl::new(account_service)),
            Some(StateRpcImpl::new(state_service)),
        )
    }

    pub fn launch_with_apis<T, A, S>(
        config: Arc<NodeConfig>,
        txpool_api: Option<T>,
        account_api: Option<A>,
        state_api: Option<S>,
    ) -> Result<(Addr<Self>, IoHandler)>
    where
        T: TxPoolApi,
        A: AccountApi,
        S: StateApi,
    {
        let mut io_handler = IoHandler::new();
        io_handler.extend_with(NodeApi::to_delegate(NodeRpcImpl::new()));
        if let Some(txpool_api) = txpool_api {
            io_handler.extend_with(TxPoolApi::to_delegate(txpool_api));
        }
        if let Some(account_api) = account_api {
            io_handler.extend_with(AccountApi::to_delegate(account_api));
        }
        if let Some(state_api) = state_api {
            io_handler.extend_with(StateApi::to_delegate(state_api));
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
    use starcoin_state_api::mock::MockChainStateService;
    use starcoin_traits::mock::MockTxPoolService;
    use starcoin_wallet_api::mock::MockWalletService;

    #[stest::test]
    async fn test_start() {
        let config = Arc::new(NodeConfig::random_for_test());
        let txpool = MockTxPoolService::new();
        let account_service = MockWalletService::new().unwrap();
        let state_service = MockChainStateService::new();
        let _rpc_actor =
            JSONRpcActor::launch(config, txpool, account_service, state_service).unwrap();
    }
}
