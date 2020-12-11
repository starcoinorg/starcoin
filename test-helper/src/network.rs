// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use network_api::messages::PeerMessage;
use network_api::{MultiaddrWithPeerId, PeerMessageHandler};
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::mocker::MockHandler;
use starcoin_service_registry::{
    RegistryAsyncService, RegistryService, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::{BlockStore, Storage};
use starcoin_types::peer_info::RpcInfo;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::sync::{Arc, Mutex};

use actix::clock::Duration;
use futures_timer::Delay;
use starcoin_network::NetworkActorService;
pub use starcoin_network::NetworkAsyncService;
use starcoin_node::network_service_factory::NetworkServiceFactory;

#[derive(Clone, Default)]
pub struct MockPeerMessageHandler {
    pub messages: Arc<Mutex<Vec<PeerMessage>>>,
}

impl PeerMessageHandler for MockPeerMessageHandler {
    fn handle_message(&self, peer_message: PeerMessage) {
        self.messages.lock().unwrap().push(peer_message);
    }
}

pub async fn build_network(
    seed: Option<MultiaddrWithPeerId>,
    rpc_service_mocker: (RpcInfo, impl MockHandler<NetworkRpcService> + 'static),
) -> Result<(
    NetworkAsyncService,
    MockPeerMessageHandler,
    Arc<NodeConfig>,
    Arc<Storage>,
    ServiceRef<RegistryService>,
)> {
    let registry = RegistryService::launch();

    let mut config = NodeConfig::random_for_test();
    if let Some(seed) = seed {
        config.network.seeds = vec![seed];
    }
    let node_config = Arc::new(config);
    let (storage, _startup_info, genesis) = Genesis::init_storage_for_test(node_config.net())?;
    registry.put_shared(genesis).await?;
    registry.put_shared(node_config.clone()).await?;
    registry.put_shared(storage.clone()).await?;
    let (rpc_info, mocker) = rpc_service_mocker;
    registry.put_shared(rpc_info).await?;
    registry.register_mocker(mocker).await?;

    registry
        .register_by_factory::<NetworkActorService, MockNetworkServiceFactory>()
        .await?;
    Delay::new(Duration::from_millis(200)).await;
    let service = registry.get_shared::<NetworkAsyncService>().await?;
    let peer_message_handle = registry.get_shared::<MockPeerMessageHandler>().await?;
    Ok((service, peer_message_handle, node_config, storage, registry))
}

pub struct MockNetworkServiceFactory;

impl ServiceFactory<NetworkActorService> for MockNetworkServiceFactory {
    fn create(ctx: &mut ServiceContext<NetworkActorService>) -> Result<NetworkActorService> {
        let rpc_info = ctx.get_shared::<RpcInfo>()?;
        let genesis = ctx.get_shared::<Genesis>()?;
        let network_rpc_service = ctx.service_ref::<NetworkRpcService>()?.clone();
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

        let chain_info = ChainInfo::new(
            config.net().chain_id(),
            genesis_hash,
            ChainStatus::new(head_block_header, head_block_info.total_difficulty),
        );
        let actor_service = NetworkActorService::new(
            config,
            chain_info,
            Some((rpc_info, network_rpc_service)),
            peer_message_handle.clone(),
        )?;
        let network_service = actor_service.network_service();
        let network_async_service = NetworkAsyncService::new(network_service, ctx.self_ref());
        ctx.put_shared(network_async_service)?;
        ctx.put_shared(peer_message_handle)?;
        Ok(actor_service)
    }
}
