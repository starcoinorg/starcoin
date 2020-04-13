// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::module::{AccountRpcImpl, DebugRpcImpl, NodeRpcImpl, StateRpcImpl, TxPoolRpcImpl};
use crate::service::RpcService;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use jsonrpc_core::IoHandler;
use starcoin_logger::prelude::*;
use starcoin_logger::LoggerHandle;
use starcoin_rpc_api::account::AccountApi;
use starcoin_rpc_api::debug::DebugApi;
use starcoin_rpc_api::{node::NodeApi, state::StateApi, txpool::TxPoolApi};
use starcoin_state_api::ChainStateAsyncService;
use starcoin_txpool_api::TxPoolAsyncService;
use starcoin_wallet_api::WalletAsyncService;
use std::sync::Arc;

pub struct RpcActor {
    config: Arc<NodeConfig>,
    io_handler: IoHandler,
    server: Option<RpcService>,
}

impl RpcActor {
    pub fn launch<TS, AS, SS>(
        config: Arc<NodeConfig>,
        txpool_service: TS,
        account_service: AS,
        state_service: SS,
        logger_handle: Option<Arc<LoggerHandle>>,
    ) -> Result<(Addr<RpcActor>, IoHandler)>
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
            logger_handle.map(|logger_handle| DebugRpcImpl::new(logger_handle)),
        )
    }

    pub fn launch_with_apis<T, A, S, D>(
        config: Arc<NodeConfig>,
        txpool_api: Option<T>,
        account_api: Option<A>,
        state_api: Option<S>,
        debug_api: Option<D>,
    ) -> Result<(Addr<Self>, IoHandler)>
    where
        T: TxPoolApi,
        A: AccountApi,
        S: StateApi,
        D: DebugApi,
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
        if let Some(debug_api) = debug_api {
            io_handler.extend_with(DebugApi::to_delegate(debug_api));
        }
        Self::launch_with_handler(config, io_handler)
    }

    pub fn launch_with_handler(
        config: Arc<NodeConfig>,
        io_handler: IoHandler,
    ) -> Result<(Addr<Self>, IoHandler)> {
        let actor = RpcActor {
            config,
            server: None,
            io_handler: io_handler.clone(),
        };
        Ok((actor.start(), io_handler))
    }

    fn do_start(&mut self) {
        let server = RpcService::new(self.config.clone(), self.io_handler.clone());
        self.server = Some(server);
    }

    fn do_stop(&mut self) {
        let server = std::mem::replace(&mut self.server, None);
        match server {
            Some(server) => server.close(),
            None => {}
        }
    }
}

impl Actor for RpcActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.do_start();
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.do_stop();
        Running::Stop
    }
}

impl Supervised for RpcActor {
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
    use starcoin_txpool_mock_service::MockTxPoolService;
    use starcoin_wallet_api::mock::MockWalletService;

    #[stest::test]
    async fn test_start() {
        let logger_handle = starcoin_logger::init_for_test();
        let config = Arc::new(NodeConfig::random_for_test());
        let txpool = MockTxPoolService::new();
        let account_service = MockWalletService::new().unwrap();
        let state_service = MockChainStateService::new();
        let _rpc_actor = RpcActor::launch(
            config,
            txpool,
            account_service,
            state_service,
            Some(logger_handle),
        )
        .unwrap();
    }
}
