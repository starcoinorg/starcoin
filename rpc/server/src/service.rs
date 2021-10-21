// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::api_registry::ApiRegistry;
use crate::extractors::{RpcExtractor, WsExtractor};
use anyhow::Result;
use futures::stream::*;
use futures::{FutureExt, StreamExt};
use jsonrpc_core::futures::channel::mpsc;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_core_client::{
    transports::{duplex, local::LocalRpc},
    RpcChannel, RpcError,
};
use jsonrpc_pubsub::Session;
use jsonrpc_server_utils::cors::AccessControlAllowOrigin;
use jsonrpc_server_utils::hosts::DomainsValidation;
use starcoin_config::{Api, ApiSet, NodeConfig};
use starcoin_logger::prelude::*;
use starcoin_rpc_api::contract_api::ContractApi;
use starcoin_rpc_api::metadata::Metadata;
use starcoin_rpc_api::network_manager::NetworkManagerApi;
use starcoin_rpc_api::node_manager::NodeManagerApi;
use starcoin_rpc_api::sync_manager::SyncManagerApi;
use starcoin_rpc_api::types::ConnectLocal;
use starcoin_rpc_api::{
    account::AccountApi, chain::ChainApi, debug::DebugApi, miner::MinerApi, node::NodeApi,
    pubsub::StarcoinPubSub, state::StateApi, txpool::TxPoolApi,
};
use starcoin_rpc_middleware::RpcMetrics;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceHandler};
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

pub struct RpcService {
    config: Arc<NodeConfig>,
    api_registry: ApiRegistry,
    ipc: Option<jsonrpc_ipc_server::Server>,
    http: Option<jsonrpc_http_server::Server>,
    tcp: Option<jsonrpc_tcp_server::Server>,
    ws: Option<jsonrpc_ws_server::Server>,
}

impl ActorService for RpcService {
    fn started(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        self.ipc = self.start_ipc()?;
        self.http = self.start_http()?;
        self.tcp = self.start_tcp()?;
        self.ws = self.start_ws()?;
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
            http: None,
            tcp: None,
            ws: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_api<C, N, NM, SM, NWM, T, A, S, D, P, M, Contract>(
        config: Arc<NodeConfig>,
        node_api: N,
        node_manager_api: Option<NM>,
        sync_manager_api: Option<SM>,
        network_manager_api: Option<NWM>,
        chain_api: Option<C>,
        txpool_api: Option<T>,
        account_api: Option<A>,
        state_api: Option<S>,
        pubsub_api: Option<P>,
        debug_api: Option<D>,
        miner_api: Option<M>,
        contract_api: Option<Contract>,
    ) -> Self
    where
        N: NodeApi,
        NM: NodeManagerApi,
        SM: SyncManagerApi,
        NWM: NetworkManagerApi,
        C: ChainApi,
        T: TxPoolApi,
        A: AccountApi,
        S: StateApi,
        P: StarcoinPubSub<Metadata = Metadata>,
        D: DebugApi,
        M: MinerApi,
        Contract: ContractApi,
    {
        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| RpcMetrics::register(registry).ok());

        let mut api_registry = ApiRegistry::new(config.rpc.api_quotas.clone(), metrics);

        api_registry.register(Api::Node, NodeApi::to_delegate(node_api));
        if let Some(node_manager_api) = node_manager_api {
            api_registry.register(
                Api::NodeManager,
                NodeManagerApi::to_delegate(node_manager_api),
            );
        }
        if let Some(sync_manager_api) = sync_manager_api {
            api_registry.register(
                Api::SyncManager,
                SyncManagerApi::to_delegate(sync_manager_api),
            )
        }
        if let Some(network_manager_api) = network_manager_api {
            api_registry.register(
                Api::NetworkManager,
                NetworkManagerApi::to_delegate(network_manager_api),
            )
        }
        if let Some(chain_api) = chain_api {
            api_registry.register(Api::Chain, ChainApi::to_delegate(chain_api));
        }
        if let Some(txpool_api) = txpool_api {
            api_registry.register(Api::TxPool, TxPoolApi::to_delegate(txpool_api));
        }
        if let Some(account_api) = account_api {
            api_registry.register(Api::Account, AccountApi::to_delegate(account_api));
        }
        if let Some(state_api) = state_api {
            api_registry.register(Api::State, StateApi::to_delegate(state_api));
        }
        if let Some(pubsub_api) = pubsub_api {
            api_registry.register(Api::PubSub, StarcoinPubSub::to_delegate(pubsub_api));
        }
        if let Some(debug_api) = debug_api {
            api_registry.register(Api::Debug, DebugApi::to_delegate(debug_api));
        }
        if let Some(miner_api) = miner_api {
            api_registry.register(Api::Miner, MinerApi::to_delegate(miner_api));
        }
        if let Some(contract_api) = contract_api {
            api_registry.register(Api::Contract, ContractApi::to_delegate(contract_api));
        }
        Self::new(config, api_registry)
    }

    fn start_ipc(&self) -> Result<Option<jsonrpc_ipc_server::Server>> {
        Ok(if self.config.rpc.ipc.disable {
            None
        } else {
            let ipc_file = self.config.rpc.get_ipc_file();
            let apis: HashSet<Api> = self.config.rpc.ipc.apis().list_apis();
            let io_handler = self.api_registry.get_apis(apis);

            info!("Ipc rpc server start at :{:?}", ipc_file);
            Some(
                jsonrpc_ipc_server::ServerBuilder::new(io_handler)
                    .session_meta_extractor(RpcExtractor::default())
                    .start(ipc_file.to_str().expect("Path to string should success."))?,
            )
        })
    }

    fn start_http(&self) -> Result<Option<jsonrpc_http_server::Server>> {
        Ok(if let Some(addr) = self.config.rpc.get_http_address() {
            let address = addr.into();
            let apis = self.config.rpc.http.apis().list_apis();
            let io_handler = self.api_registry.get_apis(apis);
            let http = jsonrpc_http_server::ServerBuilder::new(io_handler)
                .meta_extractor(RpcExtractor {
                    http_ip_headers: self.config.rpc.http.ip_headers(),
                })
                .cors(DomainsValidation::AllowOnly(vec![
                    AccessControlAllowOrigin::Null,
                    AccessControlAllowOrigin::Any,
                ]))
                .threads(self.config.rpc.http.threads())
                .max_request_body_size(self.config.rpc.http.max_request_body_size())
                .health_api(("/status", "status"))
                .start_http(&address)?;
            info!("Rpc: http server start at :{}", address);
            Some(http)
        } else {
            None
        })
    }

    fn start_tcp(&self) -> Result<Option<jsonrpc_tcp_server::Server>> {
        Ok(if let Some(addr) = self.config.rpc.get_tcp_address() {
            let address = addr.into();
            let apis = self.config.rpc.tcp.apis().list_apis();

            let io_handler = self.api_registry.get_apis(apis);
            let tcp_server = jsonrpc_tcp_server::ServerBuilder::new(io_handler)
                .session_meta_extractor(RpcExtractor::default())
                .start(&address)?;
            info!("Rpc: tcp server start at: {}", address);
            Some(tcp_server)
        } else {
            None
        })
    }

    fn start_ws(&self) -> Result<Option<jsonrpc_ws_server::Server>> {
        Ok(if let Some(addr) = self.config.rpc.get_ws_address() {
            let address = addr.into();
            let apis = self.config.rpc.ws.apis().list_apis();
            let io_handler = self.api_registry.get_apis(apis);
            let ws_server = jsonrpc_ws_server::ServerBuilder::new(io_handler)
                .session_meta_extractor(WsExtractor)
                .max_payload(self.config.rpc.ws.max_request_body_size())
                .start(&address)?;
            info!("Rpc: websocket server start at: {}", address);
            Some(ws_server)
        } else {
            None
        })
    }

    pub fn close(&mut self) {
        if let Some(ipc) = self.ipc.take() {
            ipc.close();
        }
        if let Some(http) = self.http.take() {
            http.close();
        }
        if let Some(tcp) = self.tcp.take() {
            tcp.close();
        }
        if let Some(ws) = self.ws.take() {
            ws.close();
        }
        info!("Rpc Sever is closed.");
    }
}

struct IoHandlerWrap(MetaIoHandler<Metadata>);

impl Deref for IoHandlerWrap {
    type Target = MetaIoHandler<Metadata>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Connects with pubsub.
pub fn connect_local(
    handler: MetaIoHandler<Metadata>,
) -> (
    RpcChannel,
    impl jsonrpc_core::futures::Future<Output = Result<(), RpcError>>,
) {
    let (tx, rx) = mpsc::unbounded();
    let meta = Metadata::new(Arc::new(Session::new(tx)));
    let (sink, stream) = LocalRpc::with_metadata(IoHandlerWrap(handler), meta).split();
    let stream = select(stream, rx);
    let (rpc_client, sender) = duplex(Box::pin(sink), Box::pin(stream));
    (sender, rpc_client)
}

impl ServiceHandler<Self, ConnectLocal> for RpcService {
    fn handle(&mut self, _msg: ConnectLocal, ctx: &mut ServiceContext<RpcService>) -> RpcChannel {
        let apis = ApiSet::All.list_apis();
        let io_handler = self.api_registry.get_apis(apis);
        //remove middleware.
        let mut local_io_handler = MetaIoHandler::default();
        local_io_handler.extend_with(io_handler.iter().map(|(n, f)| (n.clone(), f.clone())));
        let (rpc_channel, fut) = connect_local(local_io_handler);
        ctx.spawn(fut.map(|rs| {
            if let Err(e) = rs {
                error!("Local connect rpc error: {:?}", e);
            }
        }));
        rpc_channel
    }
}
