// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures_timer::Delay;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_storage::Storage;
use starcoin_txpool::{TxPoolActorService, TxPoolService};
use std::sync::Arc;
use std::time::Duration;

pub async fn start_txpool_with_size(
    pool_size: u64,
) -> (
    TxPoolService,
    Arc<Storage>,
    Arc<NodeConfig>,
    ServiceRef<TxPoolActorService>,
    ServiceRef<RegistryService>,
) {
    let mut config = NodeConfig::random_for_test();
    config.tx_pool.max_count = pool_size;
    let node_config = Arc::new(config);
    // let node_config = Arc::new(NodeConfig::random_for_test());

    let (storage, _startup_info, _) =
        Genesis::init_storage_for_test(node_config.net()).expect("init storage by genesis fail.");
    let registry = RegistryService::launch();
    registry.put_shared(node_config.clone()).await.unwrap();
    registry.put_shared(storage.clone()).await.unwrap();
    let bus = registry.service_ref::<BusService>().await.unwrap();
    registry.put_shared(bus).await.unwrap();

    let pool_actor = registry.register::<TxPoolActorService>().await.unwrap();
    Delay::new(Duration::from_millis(200)).await;
    let txpool_service = registry.get_shared::<TxPoolService>().await.unwrap();

    (txpool_service, storage, node_config, pool_actor, registry)
}

pub async fn start_txpool() -> (
    TxPoolService,
    Arc<Storage>,
    Arc<NodeConfig>,
    ServiceRef<TxPoolActorService>,
    ServiceRef<RegistryService>,
) {
    start_txpool_with_size(1000).await
}
