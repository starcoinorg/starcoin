use crate::TxPoolRef;
use starcoin_bus::BusActor;
use starcoin_config::{NodeConfig, TxPoolConfig};
use starcoin_genesis::Genesis;
use std::sync::Arc;
use storage::{cache_storage::CacheStorage, storage::StorageInstance, Storage};
pub fn start_txpool() -> TxPoolRef {
    let cache_storage = CacheStorage::new();
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(cache_storage)).unwrap());
    let node_config = NodeConfig::random_for_test();

    let genesis = Genesis::build(node_config.net()).unwrap();
    let startup_info = genesis.execute(storage.clone()).unwrap();
    let bus = BusActor::launch();
    let pool = TxPoolRef::start(
        TxPoolConfig::default(),
        storage.clone(),
        startup_info.get_master().clone(),
        bus,
    );

    pool
}
