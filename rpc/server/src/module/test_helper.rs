use anyhow::Result;
use starcoin_bus::BusActor;
use starcoin_chain::BlockChain;
use starcoin_config::{NodeConfig, TxPoolConfig};
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_traits::Consensus;
use std::sync::Arc;
use txpool::TxPool;

pub fn start_txpool() -> (TxPool, Arc<Storage>) {
    let cache_storage = CacheStorage::new();
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(cache_storage)).unwrap());
    let node_config = NodeConfig::random_for_test();

    let genesis = Genesis::load(node_config.net()).unwrap();
    let startup_info = genesis
        .execute_genesis_block(node_config.net(), storage.clone())
        .unwrap();
    let bus = BusActor::launch();

    let pool = TxPool::start(
        TxPoolConfig::default(),
        storage.clone(),
        *startup_info.get_master(),
        bus,
    );
    (pool, storage)
}

pub fn gen_blockchain_for_test<C: Consensus>(config: Arc<NodeConfig>) -> Result<BlockChain<C>> {
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    let genesis = Genesis::load(config.net()).unwrap();
    let startup_info = genesis.execute_genesis_block(config.net(), storage.clone())?;
    let block_chain = BlockChain::<C>::new(config, *startup_info.get_master(), storage)?;
    Ok(block_chain)
}
