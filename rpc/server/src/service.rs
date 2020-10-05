// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::api_registry::{APIType, ApiRegistry};
use crate::extractors::{RpcExtractor, WsExtractor};
use anyhow::Result;
use failure::format_err;
use futures::compat::Future01CompatExt;
use futures::FutureExt;
use jsonrpc_core::futures::sync::mpsc;
use jsonrpc_core::futures::Stream;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_core_client::{
    transports::{duplex, local::LocalRpc},
    RpcChannel, RpcError,
};
use jsonrpc_pubsub::Session;
use jsonrpc_server_utils::cors::AccessControlAllowOrigin;
use jsonrpc_server_utils::hosts::DomainsValidation;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::metadata::Metadata;
use starcoin_rpc_api::node_manager::NodeManagerApi;
use starcoin_rpc_api::types::ConnectLocal;
use starcoin_rpc_api::{
    account::AccountApi, chain::ChainApi, debug::DebugApi, dev::DevApi, miner::MinerApi,
    node::NodeApi, pubsub::StarcoinPubSub, state::StateApi, txpool::TxPoolApi,
};
use starcoin_service_registry::{ActorService, ServiceContext, ServiceHandler};
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

    pub fn new_with_api<C, N, NM, T, A, S, D, P, M, DEV>(
        config: Arc<NodeConfig>,
        node_api: N,
        node_manager_api: Option<NM>,
        chain_api: Option<C>,
        txpool_api: Option<T>,
        account_api: Option<A>,
        state_api: Option<S>,
        pubsub_api: Option<P>,
        debug_api: Option<D>,
        miner_api: Option<M>,
        dev_api: Option<DEV>,
    ) -> Self
    where
        N: NodeApi,
        NM: NodeManagerApi,
        C: ChainApi,
        T: TxPoolApi,
        A: AccountApi,
        S: StateApi,
        P: StarcoinPubSub<Metadata = Metadata>,
        D: DebugApi,
        M: MinerApi,
        DEV: DevApi,
    {
        let mut api_registry = ApiRegistry::default();

        api_registry.register(APIType::Public, NodeApi::to_delegate(node_api));
        if let Some(node_manager_api) = node_manager_api {
            api_registry.register(
                APIType::Admin,
                NodeManagerApi::to_delegate(node_manager_api),
            );
        }
        if let Some(chain_api) = chain_api {
            api_registry.register(APIType::Public, ChainApi::to_delegate(chain_api));
        }
        if let Some(txpool_api) = txpool_api {
            api_registry.register(APIType::Public, TxPoolApi::to_delegate(txpool_api));
        }
        if let Some(account_api) = account_api {
            api_registry.register(APIType::Personal, AccountApi::to_delegate(account_api));
        }
        if let Some(state_api) = state_api {
            api_registry.register(APIType::Public, StateApi::to_delegate(state_api));
        }
        if let Some(pubsub_api) = pubsub_api {
            api_registry.register(APIType::Public, StarcoinPubSub::to_delegate(pubsub_api));
        }
        if let Some(debug_api) = debug_api {
            api_registry.register(APIType::Admin, DebugApi::to_delegate(debug_api));
        }
        if let Some(miner_api) = miner_api {
            api_registry.register(APIType::Public, MinerApi::to_delegate(miner_api));
        }
        if let Some(dev_api) = dev_api {
            api_registry.register(APIType::Admin, DevApi::to_delegate(dev_api));
        }
        Self::new(config, api_registry)
    }

    #[cfg(not(windows))]
    fn start_ipc(&self) -> Result<Option<jsonrpc_ipc_server::Server>> {
        let ipc_file = self.config.rpc.get_ipc_file();
        let io_handler =
            self.api_registry
                .get_apis(&[APIType::Public, APIType::Personal, APIType::Admin]);

        info!("Ipc rpc server start at :{:?}", ipc_file);
        Ok(Some(
            jsonrpc_ipc_server::ServerBuilder::new(io_handler)
                .session_meta_extractor(RpcExtractor)
                .start(ipc_file.to_str().expect("Path to string should success."))?,
        ))
    }

    //IPC raise a error on windows: The filename, directory name, or volume label syntax is incorrect.
    #[cfg(windows)]
    fn start_ipc(&self) -> Result<Option<jsonrpc_ipc_server::Server>> {
        Ok(None)
    }

    fn start_http(&self) -> Result<Option<jsonrpc_http_server::Server>> {
        Ok(match &self.config.rpc.http_address {
            Some(address) => {
                let io_handler = self.api_registry.get_apis(&[APIType::Public]);
                let http = jsonrpc_http_server::ServerBuilder::new(io_handler)
                    .meta_extractor(RpcExtractor)
                    .cors(DomainsValidation::AllowOnly(vec![
                        AccessControlAllowOrigin::Null,
                        AccessControlAllowOrigin::Any,
                    ]))
                    .threads(self.config.rpc.threads.unwrap_or_else(num_cpus::get))
                    .max_request_body_size(self.config.rpc.max_request_body_size)
                    .health_api(("/status", "status"))
                    .start_http(address)?;
                info!("Http rpc server start at :{}", address);
                Some(http)
            }
            None => None,
        })
    }

    fn start_tcp(&self) -> Result<Option<jsonrpc_tcp_server::Server>> {
        Ok(match &self.config.rpc.tcp_address {
            Some(address) => {
                let io_handler = self.api_registry.get_apis(&[APIType::Public]);
                let tcp_server = jsonrpc_tcp_server::ServerBuilder::new(io_handler)
                    .session_meta_extractor(RpcExtractor)
                    .start(address)?;
                info!("Rpc: tcp server start at: {}", address);
                Some(tcp_server)
            }
            None => None,
        })
    }

    fn start_ws(&self) -> Result<Option<jsonrpc_ws_server::Server>> {
        Ok(match &self.config.rpc.ws_address {
            None => None,
            Some(address) => {
                let io_handler = self.api_registry.get_apis(&[APIType::Public]);
                let ws_server = jsonrpc_ws_server::ServerBuilder::new(io_handler)
                    .session_meta_extractor(WsExtractor)
                    .max_payload(self.config.rpc.max_request_body_size)
                    .start(address)?;
                info!("Rpc: websocket server start at: {}", address);
                Some(ws_server)
            }
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
    impl jsonrpc_core::futures::Future<Item = (), Error = RpcError>,
) {
    let (tx, rx) = mpsc::channel(0);
    let meta = Metadata::new(Arc::new(Session::new(tx)));
    let (sink, stream) = LocalRpc::with_metadata(IoHandlerWrap(handler), meta).split();
    let stream = stream
        .select(rx.map_err(|_| RpcError::Other(format_err!("Pubsub channel returned an error"))));
    let (rpc_client, sender) = duplex(sink, stream);
    (sender, rpc_client)
}

impl ServiceHandler<Self, ConnectLocal> for RpcService {
    fn handle(&mut self, _msg: ConnectLocal, ctx: &mut ServiceContext<RpcService>) -> RpcChannel {
        let io_handler =
            self.api_registry
                .get_apis(&[APIType::Public, APIType::Personal, APIType::Admin]);
        //remove middleware.
        let mut local_io_handler = MetaIoHandler::default();
        local_io_handler.extend_with(io_handler.iter().map(|(n, f)| (n.clone(), f.clone())));
        let (rpc_channel, fut) = connect_local(local_io_handler);
        ctx.spawn(fut.compat().map(|rs| {
            if let Err(e) = rs {
                error!("Local connect rpc error: {:?}", e);
            }
        }));
        rpc_channel
    }
}
