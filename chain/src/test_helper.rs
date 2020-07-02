use crate::BlockChain;
use anyhow::Result;
use config::NodeConfig;
use starcoin_genesis::Genesis;
use std::sync::Arc;
use storage::{cache_storage::CacheStorage, storage::StorageInstance, Storage};
use traits::Consensus;

pub fn gen_blockchain_for_test<C: Consensus>(config: Arc<NodeConfig>) -> Result<BlockChain<C>> {
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    let genesis = Genesis::load(config.net()).unwrap();
    let startup_info = genesis.execute_genesis_block(config.net(), storage.clone())?;
    let block_chain = BlockChain::<C>::new(config, *startup_info.get_master(), storage)?;
    Ok(block_chain)
}
