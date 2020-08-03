use anyhow::Result;
use starcoin_bus::BusActor;
use starcoin_chain::BlockChain;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use std::sync::Arc;
use txpool::TxPool;

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

pub fn gen_blockchain_for_test(config: Arc<NodeConfig>) -> Result<BlockChain> {
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    let genesis = Genesis::load(config.net()).unwrap();
    let startup_info = genesis.execute_genesis_block(storage.clone())?;
    let block_chain = BlockChain::new(config, *startup_info.get_master(), storage, None)?;
    Ok(block_chain)
}
