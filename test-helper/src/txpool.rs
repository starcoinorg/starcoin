// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures_timer::Delay;
use starcoin_account_service::{AccountService, AccountStorage};
use starcoin_config::NodeConfig;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_genesis::Genesis;
use starcoin_miner::{BlockBuilderService, MinerService, NewHeaderChannel, NewHeaderService};
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_storage::Storage;
use starcoin_sync::block_connector::BlockConnectorService;
use starcoin_txpool::{TxPoolActorService, TxPoolService};
use starcoin_vm2_account_service::{
    AccountService as AccountService2, AccountStorage as AccountStorage2,
};
use starcoin_vm2_storage::Storage as Storage2;
use std::sync::Arc;
use std::time::Duration;

pub async fn start_txpool_with_size(
    pool_size: u64,
) -> (
    TxPoolService,
    Arc<Storage>,
    Arc<Storage2>,
    Arc<NodeConfig>,
    ServiceRef<TxPoolActorService>,
    ServiceRef<RegistryService>,
    BlockDAG,
) {
    start_txpool_with_miner(pool_size, false).await
}

pub async fn start_txpool_with_miner(
    pool_size: u64,
    enable_miner: bool,
) -> (
    TxPoolService,
    Arc<Storage>,
    Arc<Storage2>,
    Arc<NodeConfig>,
    ServiceRef<TxPoolActorService>,
    ServiceRef<RegistryService>,
    BlockDAG,
) {
    let mut config = NodeConfig::random_for_test();
    config.tx_pool.set_max_count(pool_size);
    config.miner.disable_miner_client = Some(!enable_miner);

    let node_config = Arc::new(config);

    let (storage, storage2, _chain_info, _, _dag) =
        Genesis::init_storage_for_test(node_config.net()).expect("init storage by genesis fail.");
    let registry = RegistryService::launch();
    registry.put_shared(node_config.clone()).await.unwrap();
    registry.put_shared(storage.clone()).await.unwrap();
    registry.put_shared(storage2.clone()).await.unwrap();
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

    let config2 = starcoin_vm2_storage::db_storage::RocksdbConfig::new(
        node_config.storage.rocksdb_config().max_open_files,
        node_config.storage.rocksdb_config().max_total_wal_size,
        node_config.storage.rocksdb_config().bytes_per_sync,
        node_config.storage.rocksdb_config().wal_bytes_per_sync,
    );
    let account_storage2 = AccountStorage2::create_from_path(vault_config.dir2(), config2).unwrap();
    registry
        .put_shared::<AccountStorage2>(account_storage2.clone())
        .await
        .unwrap();
    registry.register::<AccountService2>().await.unwrap();

    let pool_actor = registry.register::<TxPoolActorService>().await.unwrap();
    Delay::new(Duration::from_secs(1)).await;
    let txpool_service = registry.get_shared::<TxPoolService>().await.unwrap();

    if enable_miner {
    	registry
            .register::<BlockConnectorService<TxPoolService>>()
            .await
            .unwrap();
        registry.put_shared(NewHeaderChannel::new()).await.unwrap();
        registry.register::<NewHeaderService>().await.unwrap();
        registry.register::<BlockBuilderService>().await.unwrap();
        registry.register::<MinerService>().await.unwrap();
    }
    //registry.register::<MinerService>().await.unwrap();
    let pool_actor = registry.register::<TxPoolActorService>().await.unwrap();
    Delay::new(Duration::from_secs(1)).await;
    let txpool_service = registry.get_shared::<TxPoolService>().await.unwrap();

    (
        txpool_service,
        storage,
        storage2,
        node_config,
        pool_actor,
        registry,
        dag,
    )
}

pub async fn start_txpool() -> (
    TxPoolService,
    Arc<Storage>,
    Arc<Storage2>,
    Arc<NodeConfig>,
    ServiceRef<TxPoolActorService>,
    ServiceRef<RegistryService>,
    BlockDAG,
) {
    start_txpool_with_size(1000).await
}
