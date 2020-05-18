use crate::{to_block_chain_collection, BlockChain, BlockChainCollection};
use anyhow::Result;
use config::NodeConfig;
use starcoin_genesis::Genesis;
use std::sync::Arc;
use storage::cache_storage::CacheStorage;
use storage::storage::StorageInstance;
use storage::Storage;
use traits::Consensus;
pub fn gen_blockchain_for_test<C: Consensus>(
    config: Arc<NodeConfig>,
) -> Result<(
    Arc<BlockChainCollection<C, Storage>>,
    BlockChain<C, Storage>,
)> {
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    let genesis = Genesis::build(config.net()).unwrap();
    let startup_info = genesis.execute(storage.clone())?;
    let collection =
        to_block_chain_collection(config.clone(), startup_info.clone(), storage.clone())?;
    let block_chain = BlockChain::<C, Storage>::new(
        config,
        *startup_info.get_master(),
        storage,
        Arc::downgrade(&collection),
    )?;
    Ok((collection, block_chain))
}
