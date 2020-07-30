use crate::BlockChain;
use anyhow::Result;
use config::NodeConfig;
use starcoin_genesis::Genesis;
use std::sync::Arc;
use storage::{cache_storage::CacheStorage, storage::StorageInstance, Storage};

pub fn gen_blockchain_for_test(config: Arc<NodeConfig>) -> Result<BlockChain> {
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    let genesis = Genesis::load(config.net()).unwrap();
    let startup_info = genesis.execute_genesis_block(storage.clone())?;
    let block_chain = BlockChain::new(config, *startup_info.get_master(), storage,None)?;
    Ok(block_chain)
}
