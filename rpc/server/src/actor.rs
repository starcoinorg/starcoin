// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::metadata::Metadata;
use crate::module::{
    AccountRpcImpl, ChainRpcImpl, DebugRpcImpl, DevRpcImpl, NodeRpcImpl, PubSubImpl, PubSubService,
    StateRpcImpl, TxPoolRpcImpl,
};
use crate::service::RpcService;
use actix::prelude::*;
use anyhow::Result;
use jsonrpc_core::{MetaIoHandler, RemoteProcedure};
use starcoin_account_api::AccountAsyncService;
use starcoin_config::NodeConfig;
use starcoin_dev::playground::PlaygroudService;
use starcoin_logger::prelude::*;
use starcoin_logger::LoggerHandle;
use starcoin_network::NetworkAsyncService;
use starcoin_rpc_api::account::AccountApi;
use starcoin_rpc_api::chain::ChainApi;
use starcoin_rpc_api::debug::DebugApi;
use starcoin_rpc_api::{
    dev::DevApi, node::NodeApi, pubsub::StarcoinPubSub, state::StateApi, txpool::TxPoolApi,
};
use starcoin_rpc_middleware::MetricMiddleware;
use starcoin_state_api::ChainStateAsyncService;
use starcoin_traits::ChainAsyncService;
use starcoin_txpool_api::TxPoolSyncService;
use std::sync::Arc;

pub struct RpcActor {
    config: Arc<NodeConfig>,
    io_handler: MetaIoHandler<Metadata, MetricMiddleware>,
    server: Option<RpcService>,
}

impl RpcActor {
    pub fn launch<CS, TS, AS, SS>(
        config: Arc<NodeConfig>,
        txpool_service: TS,
        chain_service: CS,
        account_service: AS,
        state_service: SS,
        dev_playground_service: Option<PlaygroudService>,
        pubsub_service: Option<PubSubService>,
        //TODO after network async service provide trait, remove Option.
        network_service: Option<NetworkAsyncService>,
        logger_handle: Option<Arc<LoggerHandle>>,
    ) -> Result<(Addr<RpcActor>, MetaIoHandler<Metadata, MetricMiddleware>)>
    where
        CS: ChainAsyncService + 'static,
        TS: TxPoolSyncService + 'static,
        AS: AccountAsyncService + 'static,
        SS: ChainStateAsyncService + 'static,
    {
        let config_clone = config.clone();
        let mut io_handler = Self::extend_apis(
            NodeRpcImpl::new(config.clone(), network_service),
            Some(ChainRpcImpl::new(chain_service)),
            Some(TxPoolRpcImpl::new(txpool_service)),
            Some(AccountRpcImpl::new(account_service)),
            Some(StateRpcImpl::new(state_service.clone())),
            pubsub_service.map(PubSubImpl::new),
            logger_handle.map(|logger_handle| DebugRpcImpl::new(config_clone, logger_handle)),
        )?;

        if let Some(dev_playgroud) = dev_playground_service {
            io_handler.extend_with(DevApi::to_delegate(DevRpcImpl::new(
                state_service,
                dev_playgroud,
            )));
        }

        Self::launch_with_handler(config, io_handler)
    }

    pub fn extend_apis<C, N, T, A, S, D, P>(
        node_api: N,
        chain_api: Option<C>,
        txpool_api: Option<T>,
        account_api: Option<A>,
        state_api: Option<S>,
        pubsub_api: Option<P>,
        debug_api: Option<D>,
    ) -> Result<MetaIoHandler<Metadata, MetricMiddleware>>
    where
        N: NodeApi,
        C: ChainApi,
        T: TxPoolApi,
        A: AccountApi,
        S: StateApi,
        P: StarcoinPubSub<Metadata = Metadata>,
        D: DebugApi,
    {
        let mut io_handler =
            MetaIoHandler::<Metadata, MetricMiddleware>::with_middleware(MetricMiddleware);
        io_handler.extend_with(NodeApi::to_delegate(node_api));
        if let Some(chain_api) = chain_api {
            io_handler.extend_with(ChainApi::to_delegate(chain_api));
        }
        if let Some(txpool_api) = txpool_api {
            io_handler.extend_with(TxPoolApi::to_delegate(txpool_api));
        }
        if let Some(account_api) = account_api {
            io_handler.extend_with(AccountApi::to_delegate(account_api));
        }
        if let Some(state_api) = state_api {
            io_handler.extend_with(StateApi::to_delegate(state_api));
        }
        if let Some(pubsub_api) = pubsub_api {
            io_handler.extend_with(StarcoinPubSub::to_delegate(pubsub_api));
        }
        if let Some(debug_api) = debug_api {
            io_handler.extend_with(DebugApi::to_delegate(debug_api));
        }
        Ok(io_handler)
        // Self::launch_with_handler(config, io_handler)
    }

    pub fn launch_with_handler(
        config: Arc<NodeConfig>,
        io_handler: MetaIoHandler<Metadata, MetricMiddleware>,
    ) -> Result<(Addr<Self>, MetaIoHandler<Metadata, MetricMiddleware>)> {
        let actor = RpcActor {
            config,
            server: None,
            io_handler: io_handler.clone(),
        };
        Ok((actor.start(), io_handler))
    }

    pub fn launch_with_method<F>(
        config: Arc<NodeConfig>,
        method: F,
    ) -> Result<(Addr<Self>, MetaIoHandler<Metadata, MetricMiddleware>)>
    where
        F: IntoIterator<Item = (String, RemoteProcedure<Metadata>)>,
    {
        let mut io_handler =
            MetaIoHandler::<Metadata, MetricMiddleware>::with_middleware(MetricMiddleware);
        io_handler.extend_with(method);
        Self::launch_with_handler(config, io_handler)
    }

    fn do_start(&mut self) {
        let server = RpcService::new(self.config.clone(), self.io_handler.clone());
        self.server = Some(server);
    }

    fn do_stop(&mut self) {
        let server = std::mem::replace(&mut self.server, None);
        if let Some(server) = server {
            server.close()
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
    use starcoin_account_api::mock::MockAccountService;
    use starcoin_chain::mock::mock_chain_service::MockChainService;
    use starcoin_state_api::mock::MockChainStateService;
    use starcoin_state_tree::mock::MockStateNodeStore;
    use starcoin_txpool_mock_service::MockTxPoolService;
    #[stest::test]
    async fn test_start() {
        let logger_handle = starcoin_logger::init_for_test();
        let config = Arc::new(NodeConfig::random_for_test());
        let txpool = MockTxPoolService::new();
        let account_service = MockAccountService::new().unwrap();
        let state_service = MockChainStateService::new();
        let chain_service = MockChainService::default();
        let playground_service = PlaygroudService::new(Arc::new(MockStateNodeStore::new()));
        let _rpc_actor = RpcActor::launch(
            config,
            txpool,
            chain_service,
            account_service,
            state_service,
            Some(playground_service),
            None,
            None,
            Some(logger_handle),
        )
        .unwrap();
    }
}
