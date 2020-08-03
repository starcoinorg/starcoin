use crate::TxPool;
use starcoin_bus::BusActor;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use std::sync::Arc;
use storage::{cache_storage::CacheStorage, storage::StorageInstance, Storage};

pub fn start_txpool() -> (TxPool, Arc<Storage>, Arc<NodeConfig>) {
    let cache_storage = CacheStorage::new();
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(cache_storage)).unwrap());
    let node_config = Arc::new(NodeConfig::random_for_test());

    let genesis = Genesis::load(node_config.net()).unwrap();
    let startup_info = genesis.execute_genesis_block(storage.clone()).unwrap();
    let bus = BusActor::launch();

    let pool = TxPool::start(
        node_config.clone(),
        storage.clone(),
        *startup_info.get_master(),
        bus,
    );
    (pool, storage, node_config)
}
