// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::api_registry::ApiRegistry;
use anyhow::Result;
use futures::FutureExt;
use jsonrpsee::async_client::Client;
use starcoin_config::{Api, ApiSet, NodeConfig};
use starcoin_logger::prelude::*;
use starcoin_rpc_api::types::ConnectLocal;
use starcoin_rpc_api::{
    account::{account_methods, AccountApi},
    chain::{chain_methods, ChainApi},
    contract_api::{contract_methods, ContractApi},
    debug::{debug_methods, DebugApi},
    miner::{miner_methods, MinerApi},
    network_manager::{network_manager_methods, NetworkManagerApi},
    node::{node_methods, NodeApi},
    node_manager::{node_manager_methods, NodeManagerApi},
    state::{state_methods, StateApi},
    sync_manager::{sync_manager_methods, SyncManagerApi},
    txpool::{txpool_methods, TxPoolApi},
};
use starcoin_rpc_middleware::RpcMetrics;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceHandler};
use starcoin_vm2_rpc_api::{
    account_api::{account_methods as account2_methods, AccountApi as AccountApi2},
    contract_api::{contract_methods as contract2_methods, ContractApi as ContractApi2},
    state_api::{state_methods as state2_methods, StateApi as StateApi2},
};
use std::collections::HashSet;
use std::sync::Arc;

pub struct RpcService {
    config: Arc<NodeConfig>,
    api_registry: ApiRegistry,
    ipc: Option<jsonrpsee::server::ServerHandle>,
}

impl ActorService for RpcService {
    fn started(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        self.ipc = self.start_ipc()?;
        Ok(())
    }

    fn stopped(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        self.close();
        Ok(())
    }
}

impl RpcService {
    pub fn new(config: Arc<NodeConfig>, api_registry: ApiRegistry) -> Self {
        Self {
            config,
            api_registry,
            ipc: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_api<C, N, NM, SM, NWM, T, A, A2, S, S2, D, M, Contract, Contract2>(
        config: Arc<NodeConfig>,
        node_api: N,
        node_manager_api: Option<NM>,
        sync_manager_api: Option<SM>,
        network_manager_api: Option<NWM>,
        chain_api: Option<C>,
        txpool_api: Option<T>,
        account_api: Option<A>,
        state_api: Option<S>,
        _pubsub_api: Option<()>,
        debug_api: Option<D>,
        miner_api: Option<M>,
        contract_api: Option<Contract>,
        account_api2: Option<A2>,
        state_api2: Option<S2>,
        contract_api2: Option<Contract2>,
    ) -> Self
    where
        N: NodeApi + Send + Sync + 'static,
        NM: NodeManagerApi + Send + Sync + 'static,
        SM: SyncManagerApi + Send + Sync + 'static,
        NWM: NetworkManagerApi + Send + Sync + 'static,
        C: ChainApi + Send + Sync + 'static,
        T: TxPoolApi + Send + Sync + 'static,
        A: AccountApi + Send + Sync + 'static,
        A2: AccountApi2 + Send + Sync + 'static,
        S: StateApi + Send + Sync + 'static,
        S2: StateApi2 + Send + Sync + 'static,
        D: DebugApi + Send + Sync + 'static,
        M: MinerApi + Send + Sync + 'static,
        Contract: ContractApi + Send + Sync + 'static,
        Contract2: ContractApi2 + Send + Sync + 'static,
    {
        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| RpcMetrics::register(registry).ok());

        let mut api_registry = ApiRegistry::new(config.rpc.api_quotas.clone(), metrics);

        api_registry
            .register(Api::Node, node_methods(node_api).expect("register node methods"))
            .expect("merge node methods");

        if let Some(api) = node_manager_api {
            api_registry
                .register(
                    Api::NodeManager,
                    node_manager_methods(api).expect("register node_manager methods"),
                )
                .expect("merge node_manager methods");
        }
        if let Some(api) = sync_manager_api {
            api_registry
                .register(
                    Api::SyncManager,
                    sync_manager_methods(api).expect("register sync methods"),
                )
                .expect("merge sync methods");
        }
        if let Some(api) = network_manager_api {
            api_registry
                .register(
                    Api::NetworkManager,
                    network_manager_methods(api).expect("register network_manager methods"),
                )
                .expect("merge network_manager methods");
        }
        if let Some(api) = chain_api {
            api_registry
                .register(Api::Chain, chain_methods(api).expect("register chain methods"))
                .expect("merge chain methods");
        }
        if let Some(api) = txpool_api {
            api_registry
                .register(Api::TxPool, txpool_methods(api).expect("register txpool methods"))
                .expect("merge txpool methods");
        }
        if let Some(api) = account_api {
            api_registry
                .register(Api::Account, account_methods(api).expect("register account methods"))
                .expect("merge account methods");
        }
        if let Some(api) = state_api {
            api_registry
                .register(Api::State, state_methods(api).expect("register state methods"))
                .expect("merge state methods");
        }
        if let Some(api) = debug_api {
            api_registry
                .register(Api::Debug, debug_methods(api).expect("register debug methods"))
                .expect("merge debug methods");
        }
        if let Some(api) = miner_api {
            api_registry
                .register(Api::Miner, miner_methods(api).expect("register miner methods"))
                .expect("merge miner methods");
        }
        if let Some(api) = contract_api {
            api_registry
                .register(
                    Api::Contract,
                    contract_methods(api).expect("register contract methods"),
                )
                .expect("merge contract methods");
        }
        if let Some(api) = account_api2 {
            api_registry
                .register(Api::Account2, account2_methods(api).expect("register account2 methods"))
                .expect("merge account2 methods");
        }
        if let Some(api) = state_api2 {
            api_registry
                .register(Api::State2, state2_methods(api).expect("register state2 methods"))
                .expect("merge state2 methods");
        }
        if let Some(api) = contract_api2 {
            api_registry
                .register(
                    Api::Contract2,
                    contract2_methods(api).expect("register contract2 methods"),
                )
                .expect("merge contract2 methods");
        }

        Self::new(config, api_registry)
    }

    fn start_ipc(&self) -> Result<Option<jsonrpsee::server::ServerHandle>> {
        Ok(if self.config.rpc.ipc.disable {
            None
        } else {
            let ipc_file = self.config.rpc.get_ipc_file();
            let apis: HashSet<Api> = self.config.rpc.ipc.apis().list_apis();
            let methods = self.api_registry.get_apis(apis)?;

            info!("Ipc rpc server start at :{:?}", ipc_file);
            let server = starcoin_rpc_ipc::server::Builder::default()
                .build(ipc_file.to_str().expect("Path to string should success.").to_string());
            Some(futures::executor::block_on(server.start(methods))?)
        })
    }

    pub fn close(&mut self) {
        if let Some(ipc) = self.ipc.take() {
            drop(ipc);
        }
        info!("Rpc Sever is closed.");
    }
}

impl ServiceHandler<Self, ConnectLocal> for RpcService {
    fn handle(&mut self, _msg: ConnectLocal, ctx: &mut ServiceContext<RpcService>) -> Client {
        let apis = ApiSet::All.list_apis();
        let methods = self
            .api_registry
            .get_apis(apis)
            .expect("collect local rpc methods");
        let (client, fut) = starcoin_rpc_local::connect_local(methods);
        ctx.spawn(fut.map(|rs| {
            if let Err(e) = rs {
                error!("Local connect rpc error: {:?}", e);
            }
        }));
        client
    }
}
