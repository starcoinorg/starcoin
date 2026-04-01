// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::api_registry::ApiRegistry;
use crate::metadata_middleware::HttpMetadataLayer;
use crate::module::{pubsub_methods, PubSubImpl};
use crate::rate_limit_middleware::JsonApiRateLimitLayer;
use anyhow::{anyhow, Result};
use futures::FutureExt;
use jsonrpsee::async_client::Client;
use jsonrpsee::core::middleware::RpcServiceBuilder;
use jsonrpsee::server::{ServerBuilder, ServerConfigBuilder, ServerHandle};
use starcoin_config::{Api, ApiSet, NodeConfig};
use starcoin_logger::prelude::*;
use starcoin_rpc_api::types::ConnectLocal;
use starcoin_rpc_api::{
    account::{account_methods, AccountApiServer},
    chain::{chain_methods, ChainApiServer},
    contract_api::{contract_methods, ContractApiServer},
    debug::{debug_methods, DebugApiServer},
    miner::{miner_methods, MinerApiServer},
    network_manager::{network_manager_methods, NetworkManagerApiServer},
    node::{node_methods, NodeApiServer},
    node_manager::{node_manager_methods, NodeManagerApiServer},
    state::{state_methods, StateApiServer},
    sync_manager::{sync_manager_methods, SyncManagerApiServer},
    txpool::{txpool_methods, TxPoolApiServer},
};
use starcoin_rpc_middleware::{MetricMiddleware, RpcMetrics};
use starcoin_service_registry::{ActorService, ServiceContext, ServiceHandler};
use starcoin_vm2_rpc_api::{
    account_api::{account_methods as account2_methods, AccountApiServer as AccountApiServer2},
    contract_api::{
        contract_methods as contract2_methods, ContractApiServer as ContractApiServer2,
    },
    state_api::{state_methods as state2_methods, StateApiServer as StateApiServer2},
};
use std::collections::HashSet;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::mpsc::sync_channel;
use std::sync::Arc;

pub struct RpcService {
    config: Arc<NodeConfig>,
    api_registry: ApiRegistry,
    ipc: Option<ServerHandle>,
    http: Option<ServerHandle>,
    ws: Option<ServerHandle>,
    rpc_runtime: tokio::runtime::Runtime,
}

impl ActorService for RpcService {
    fn started(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        info!(
            "RpcService endpoint config: http={:?}, ws={:?}, tcp={:?}, ipc_disable={}",
            self.config.rpc.get_http_address(),
            self.config.rpc.get_ws_address(),
            self.config.rpc.get_tcp_address(),
            self.config.rpc.ipc.disable
        );
        self.http = self.start_http().map_err(|e| {
            error!("Failed to start rpc http endpoint: {:?}", e);
            e
        })?;
        self.ws = self.start_ws().map_err(|e| {
            error!("Failed to start rpc websocket endpoint: {:?}", e);
            e
        })?;
        self.warn_tcp_unsupported();
        self.ipc = self.start_ipc().map_err(|e| {
            error!("Failed to start rpc ipc endpoint: {:?}", e);
            e
        })?;
        Ok(())
    }

    fn stopped(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        self.close();
        Ok(())
    }
}

impl RpcService {
    pub fn new(config: Arc<NodeConfig>, api_registry: ApiRegistry) -> Self {
        let rpc_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .thread_name("starcoin-rpc-runtime")
            .build()
            .expect("failed to build rpc runtime");
        Self {
            config,
            api_registry,
            ipc: None,
            http: None,
            ws: None,
            rpc_runtime,
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
        pubsub_api: Option<PubSubImpl>,
        debug_api: Option<D>,
        miner_api: Option<M>,
        contract_api: Option<Contract>,
        account_api2: Option<A2>,
        state_api2: Option<S2>,
        contract_api2: Option<Contract2>,
    ) -> Self
    where
        N: NodeApiServer + Send + Sync + 'static,
        NM: NodeManagerApiServer + Send + Sync + 'static,
        SM: SyncManagerApiServer + Send + Sync + 'static,
        NWM: NetworkManagerApiServer + Send + Sync + 'static,
        C: ChainApiServer + Send + Sync + 'static,
        T: TxPoolApiServer + Send + Sync + 'static,
        A: AccountApiServer + Send + Sync + 'static,
        A2: AccountApiServer2 + Send + Sync + 'static,
        S: StateApiServer + Send + Sync + 'static,
        S2: StateApiServer2 + Send + Sync + 'static,
        D: DebugApiServer + Send + Sync + 'static,
        M: MinerApiServer + Send + Sync + 'static,
        Contract: ContractApiServer + Send + Sync + 'static,
        Contract2: ContractApiServer2 + Send + Sync + 'static,
    {
        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| RpcMetrics::register(registry).ok());

        let mut api_registry = ApiRegistry::new(config.rpc.api_quotas.clone(), metrics);

        api_registry
            .register(
                Api::Node,
                node_methods(node_api).expect("register node methods"),
            )
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
                .register(
                    Api::Chain,
                    chain_methods(api).expect("register chain methods"),
                )
                .expect("merge chain methods");
        }
        if let Some(api) = txpool_api {
            api_registry
                .register(
                    Api::TxPool,
                    txpool_methods(api).expect("register txpool methods"),
                )
                .expect("merge txpool methods");
        }
        if let Some(api) = account_api {
            api_registry
                .register(
                    Api::Account,
                    account_methods(api).expect("register account methods"),
                )
                .expect("merge account methods");
        }
        if let Some(api) = state_api {
            api_registry
                .register(
                    Api::State,
                    state_methods(api).expect("register state methods"),
                )
                .expect("merge state methods");
        }
        if let Some(api) = pubsub_api {
            api_registry
                .register(
                    Api::PubSub,
                    pubsub_methods(api).expect("register pubsub methods"),
                )
                .expect("merge pubsub methods");
        }
        if let Some(api) = debug_api {
            api_registry
                .register(
                    Api::Debug,
                    debug_methods(api).expect("register debug methods"),
                )
                .expect("merge debug methods");
        }
        if let Some(api) = miner_api {
            api_registry
                .register(
                    Api::Miner,
                    miner_methods(api).expect("register miner methods"),
                )
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
                .register(
                    Api::Account2,
                    account2_methods(api).expect("register account2 methods"),
                )
                .expect("merge account2 methods");
        }
        if let Some(api) = state_api2 {
            api_registry
                .register(
                    Api::State2,
                    state2_methods(api).expect("register state2 methods"),
                )
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

    fn start_ipc(&self) -> Result<Option<ServerHandle>> {
        Ok(if self.config.rpc.ipc.disable {
            None
        } else {
            let ipc_file = self.config.rpc.get_ipc_file();
            let apis: HashSet<Api> = self.config.rpc.ipc.apis().list_apis();
            let methods = self.api_registry.get_apis(apis)?;
            let rpc_middleware = starcoin_rpc_ipc::server::RpcServiceBuilder::new()
                .layer_fn({
                    let metrics = self.api_registry.metrics();
                    move |service| MetricMiddleware::new(service, metrics.clone())
                })
                .layer(JsonApiRateLimitLayer::from_config(
                    self.api_registry.quotas(),
                ));

            info!("Ipc rpc server start at :{:?}", ipc_file);
            let server = starcoin_rpc_ipc::server::Builder::default()
                .set_rpc_middleware(rpc_middleware)
                .custom_tokio_runtime(self.rpc_runtime.handle().clone())
                .build(
                    ipc_file
                        .to_str()
                        .expect("Path to string should success.")
                        .to_string(),
                );
            Some(self.run_on_rpc_runtime(async move {
                server.start(methods).await.map_err(anyhow::Error::from)
            })?)
        })
    }

    fn start_http(&self) -> Result<Option<ServerHandle>> {
        Ok(if let Some(addr) = self.config.rpc.get_http_address() {
            let apis: HashSet<Api> = self.config.rpc.http.apis().list_apis();
            let methods = self.api_registry.get_apis(apis)?;
            let socket_addr: SocketAddr = addr.clone().into();
            let rpc_middleware = RpcServiceBuilder::new()
                .layer_fn({
                    let metrics = self.api_registry.metrics();
                    move |service| MetricMiddleware::new(service, metrics.clone())
                })
                .layer(JsonApiRateLimitLayer::from_config(
                    self.api_registry.quotas(),
                ));
            let http_middleware = tower::ServiceBuilder::new()
                .layer(HttpMetadataLayer::new(self.config.rpc.http.ip_headers()));

            if self.config.rpc.http.threads.is_some() {
                warn!("jsonrpsee http server ignores rpc.http.threads setting");
            }
            if self.config.rpc.http._unsupported_rpc_protocols().is_some() {
                warn!("jsonrpsee http server ignores rpc.http.unsupported_rpc_protocols");
            }

            let cfg = ServerConfigBuilder::default()
                .http_only()
                .max_request_body_size(
                    self.config
                        .rpc
                        .http
                        .max_request_body_size()
                        .min(u32::MAX as usize) as u32,
                )
                .build();
            let server = self.run_on_rpc_runtime(async move {
                ServerBuilder::with_config(cfg)
                    .set_http_middleware(http_middleware)
                    .set_rpc_middleware(rpc_middleware)
                    .build(socket_addr)
                    .await
                    .map_err(anyhow::Error::from)
            })?;

            info!("Rpc: http server start at: {}", addr);
            Some(server.start(methods))
        } else {
            None
        })
    }

    fn start_ws(&self) -> Result<Option<ServerHandle>> {
        Ok(if let Some(addr) = self.config.rpc.get_ws_address() {
            let apis: HashSet<Api> = self.config.rpc.ws.apis().list_apis();
            let methods = self.api_registry.get_apis(apis)?;
            let socket_addr: SocketAddr = addr.clone().into();
            let rpc_middleware = RpcServiceBuilder::new()
                .layer_fn({
                    let metrics = self.api_registry.metrics();
                    move |service| MetricMiddleware::new(service, metrics.clone())
                })
                .layer(JsonApiRateLimitLayer::from_config(
                    self.api_registry.quotas(),
                ));
            let cfg = ServerConfigBuilder::default()
                .ws_only()
                .max_request_body_size(
                    self.config
                        .rpc
                        .ws
                        .max_request_body_size()
                        .min(u32::MAX as usize) as u32,
                )
                .build();
            let server = self.run_on_rpc_runtime(async move {
                ServerBuilder::with_config(cfg)
                    .set_rpc_middleware(rpc_middleware)
                    .build(socket_addr)
                    .await
                    .map_err(anyhow::Error::from)
            })?;

            info!("Rpc: websocket server start at: {}", addr);
            Some(server.start(methods))
        } else {
            None
        })
    }

    fn warn_tcp_unsupported(&self) {
        if self.config.rpc.get_tcp_address().is_some() {
            warn!(
                "Rpc: tcp endpoint is configured but not supported by current jsonrpsee server backend"
            );
        }
    }

    fn run_on_rpc_runtime<F, T>(&self, fut: F) -> Result<T>
    where
        F: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = sync_channel(1);
        self.rpc_runtime.handle().spawn(async move {
            let _ = tx.send(fut.await);
        });
        rx.recv()
            .map_err(|e| anyhow!("rpc runtime startup task canceled: {e}"))?
    }

    pub fn close(&mut self) {
        if let Some(ipc) = self.ipc.take() {
            if let Err(err) = ipc.stop() {
                debug!("Rpc ipc server already stopped: {:?}", err);
            }
        }
        if let Some(http) = self.http.take() {
            if let Err(err) = http.stop() {
                debug!("Rpc http server already stopped: {:?}", err);
            }
        }
        if let Some(ws) = self.ws.take() {
            if let Err(err) = ws.stop() {
                debug!("Rpc ws server already stopped: {:?}", err);
            }
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
