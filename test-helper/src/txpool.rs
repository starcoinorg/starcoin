// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures_timer::Delay;
use starcoin_account_service::{AccountService, AccountStorage};
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_miner::{BlockBuilderService, MinerService};
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
    start_txpool_with_miner(pool_size, false).await
}

pub async fn start_txpool_with_miner(
    pool_size: u64,
    enable_miner: bool,
) -> (
    TxPoolService,
    Arc<Storage>,
    Arc<NodeConfig>,
    ServiceRef<TxPoolActorService>,
    ServiceRef<RegistryService>,
) {
    let mut config = NodeConfig::random_for_test();
    config.tx_pool.set_max_count(pool_size);
    config.miner.disable_miner_client = Some(!enable_miner);

    let node_config = Arc::new(config);

    let (storage, _chain_info, _) =
        Genesis::init_storage_for_test(node_config.net()).expect("init storage by genesis fail.");
    let registry = RegistryService::launch();
    registry.put_shared(node_config.clone()).await.unwrap();
    registry.put_shared(storage.clone()).await.unwrap();
    let bus = registry.service_ref::<BusService>().await.unwrap();
    registry.put_shared(bus).await.unwrap();

    let vault_config = &node_config.vault;
    let account_storage =
        AccountStorage::create_from_path(vault_config.dir(), node_config.storage.rocksdb_config())
            .unwrap();
    registry
        .put_shared::<AccountStorage>(account_storage.clone())
        .await
        .unwrap();
    registry.register::<AccountService>().await.unwrap();

    if enable_miner {
        registry.register::<BlockBuilderService>().await.unwrap();
        registry.register::<MinerService>().await.unwrap();
    }
    //registry.register::<MinerService>().await.unwrap();
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
