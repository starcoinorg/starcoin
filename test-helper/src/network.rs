// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use futures_timer::Delay;
use network_api::messages::PeerMessage;
use network_api::{MultiaddrWithPeerId, PeerMessageHandler};
use network_p2p_types::{OutgoingResponse, ProtocolRequest};
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_network::NetworkActorService;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::mocker::MockHandler;
use starcoin_service_registry::{
    RegistryAsyncService, RegistryService, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::{BlockStore, Storage};
use starcoin_types::peer_info::{PeerId, RpcInfo};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::any::Any;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
pub use starcoin_network::NetworkServiceRef;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::SyncStatusChangeEvent;

#[derive(Clone, Default)]
pub struct MockPeerMessageHandler {
    messages: Arc<Mutex<Vec<PeerMessage>>>,
    senders: Arc<Mutex<Vec<UnboundedSender<PeerMessage>>>>,
}

impl MockPeerMessageHandler {
    pub fn channel(&self) -> UnboundedReceiver<PeerMessage> {
        let (sender, receiver) = unbounded::<PeerMessage>();
        self.senders.lock().unwrap().push(sender);
        receiver
    }
    pub fn messages(&self) -> Vec<PeerMessage> {
        self.messages.lock().unwrap().clone()
    }
}

impl PeerMessageHandler for MockPeerMessageHandler {
    fn handle_message(&self, peer_message: PeerMessage) {
        for sender in self.senders.lock().unwrap().iter() {
            sender.unbounded_send(peer_message.clone()).unwrap();
        }
        self.messages.lock().unwrap().push(peer_message);
    }
}

pub struct MockRpcHandler {
    rpc_fn: RpcFn,
}

impl MockRpcHandler {
    pub fn new<F>(rpc_fn: F) -> Self
    where
        F: Fn(Cow<'static, str>, PeerId, Vec<u8>) -> Vec<u8> + Send + 'static,
    {
        Self {
            rpc_fn: Box::new(rpc_fn),
        }
    }

    pub fn echo() -> Self {
        Self::new(echo)
    }
}

impl MockHandler<NetworkRpcService> for MockRpcHandler {
    fn handle(
        &mut self,
        _r: Box<dyn Any>,
        _ctx: &mut ServiceContext<NetworkRpcService>,
    ) -> Box<dyn Any> {
        unreachable!()
    }

    fn handle_event(&mut self, msg: Box<dyn Any>, _ctx: &mut ServiceContext<NetworkRpcService>) {
        let req = msg.downcast::<ProtocolRequest>().unwrap();
        debug!("MockRpcHandler handle request: {:?}", req);
        let resp = (self.rpc_fn)(req.protocol, req.request.peer.into(), req.request.payload);
        req.request
            .pending_response
            .send(OutgoingResponse {
                result: Ok(resp),
                reputation_changes: vec![],
            })
            .unwrap();
    }
}

pub type RpcFn = Box<dyn Fn(Cow<'static, str>, PeerId, Vec<u8>) -> Vec<u8> + Send>;

pub fn echo(_protocol: Cow<'static, str>, _peer_id: PeerId, request: Vec<u8>) -> Vec<u8> {
    request
}

pub struct TestNetworkService {
    pub service_ref: NetworkServiceRef,
    pub message_handler: MockPeerMessageHandler,
    pub config: Arc<NodeConfig>,
    pub registry: ServiceRef<RegistryService>,
}

impl TestNetworkService {
    pub fn peer_id(&self) -> PeerId {
        self.config.network.self_peer_id()
    }
}

pub async fn build_network_pair() -> Result<(TestNetworkService, TestNetworkService)> {
    let mut nodes = build_network_cluster(2).await?;
    let second = nodes.pop().unwrap();
    let first = nodes.pop().unwrap();
    Ok((first, second))
}

pub async fn build_network_cluster(n: usize) -> Result<Vec<TestNetworkService>> {
    let seed_service = build_network(None, None).await?;
    let seed = seed_service.config.network.self_address();
    let mut nodes = vec![seed_service];
    for _i in 1..n {
        let service = build_network(Some(seed.clone()), None).await?;
        nodes.push(service);
    }
    Ok(nodes)
}

pub async fn build_network_with_config(
    node_config: Arc<NodeConfig>,
    rpc_service_mocker: Option<(RpcInfo, MockRpcHandler)>,
) -> Result<TestNetworkService> {
    let registry = RegistryService::launch();
    let (storage, _chain_info, genesis) = Genesis::init_storage_for_test(node_config.net())?;
    registry.put_shared(genesis).await?;
    registry.put_shared(node_config.clone()).await?;
    registry.put_shared(storage.clone()).await?;
    if let Some((rpc_info, mocker)) = rpc_service_mocker {
        registry.put_shared(rpc_info).await?;
        registry.register_mocker(mocker).await?;
    }
    registry
        .register_by_factory::<NetworkActorService, MockNetworkServiceFactory>()
        .await?;
    Delay::new(Duration::from_millis(200)).await;
    let service_ref = registry.get_shared::<NetworkServiceRef>().await?;
    let message_handler = registry.get_shared::<MockPeerMessageHandler>().await?;
    Ok(TestNetworkService {
        service_ref,
        message_handler,
        config: node_config,
        registry,
    })
}

pub async fn build_network(
    seed: Option<MultiaddrWithPeerId>,
    rpc_service_mocker: Option<(RpcInfo, MockRpcHandler)>,
) -> Result<TestNetworkService> {
    let mut config = NodeConfig::random_for_test();
    if let Some(seed) = seed {
        config.network.seeds = vec![seed].into();
    }
    build_network_with_config(Arc::new(config), rpc_service_mocker).await
}

pub struct MockNetworkServiceFactory;

impl ServiceFactory<NetworkActorService> for MockNetworkServiceFactory {
    fn create(ctx: &mut ServiceContext<NetworkActorService>) -> Result<NetworkActorService> {
        let genesis = ctx.get_shared::<Genesis>()?;
        let rpc_service_opt = ctx.service_ref_opt::<NetworkRpcService>()?.cloned();
        let rpc = match rpc_service_opt {
            Some(rpc_service) => Some((ctx.get_shared::<RpcInfo>()?, rpc_service)),
            None => None,
        };
        let peer_message_handle = MockPeerMessageHandler::default();
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;

        let genesis_hash = genesis.block().header().id();
        let startup_info = storage.get_startup_info()?.unwrap();
        let head_block_hash = startup_info.main;
        let head_block_header = storage
            .get_block_header_by_hash(head_block_hash)?
            .ok_or_else(|| format_err!("can't get block by hash {}", head_block_hash))?;
        let head_block_info = storage
            .get_block_info(head_block_hash)?
            .ok_or_else(|| format_err!("can't get block info by hash {}", head_block_hash))?;
        let chain_status = ChainStatus::new(head_block_header, head_block_info);
        let chain_info =
            ChainInfo::new(config.net().chain_id(), genesis_hash, chain_status.clone());
        let actor_service =
            NetworkActorService::new(config, chain_info, rpc, peer_message_handle.clone())?;
        let network_service = actor_service.network_service();
        let network_async_service = NetworkServiceRef::new(network_service, ctx.self_ref());
        // set self sync status to synced for test.
        let mut sync_status = SyncStatus::new(chain_status);
        sync_status.sync_done();
        ctx.notify(SyncStatusChangeEvent(sync_status));

        ctx.put_shared(network_async_service)?;
        ctx.put_shared(peer_message_handle)?;
        Ok(actor_service)
    }
}
