// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libp2p::core::Multiaddr;
use starcoin_bus::BusActor;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::mocker::MockHandler;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_storage::Storage;
use std::sync::Arc;

pub use starcoin_network::NetworkAsyncService;

pub async fn build_network(
    seed: Option<Multiaddr>,
    rpc_service_mocker: Option<impl MockHandler<NetworkRpcService> + 'static>,
) -> Result<(
    NetworkAsyncService,
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
    let (storage, _, genesis_hash) = Genesis::init_storage_for_test(node_config.net())?;

    registry.put_shared(node_config.clone()).await?;
    registry.put_shared(storage.clone()).await?;

    let new_bus = registry.service_ref::<BusService>().await?;
    let bus = BusActor::launch2(new_bus);
    registry.put_shared(bus.clone()).await?;
    let network_rpc_service = if let Some(mocker) = rpc_service_mocker {
        registry.registry_mocker(mocker).await?
    } else {
        registry.registry::<NetworkRpcService>().await?
    };

    Ok((
        NetworkAsyncService::start(
            node_config.clone(),
            genesis_hash,
            bus,
            storage.clone(),
            network_rpc_service,
        )?,
        node_config,
        storage,
        registry,
    ))
}
