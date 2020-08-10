use anyhow::Result;
use starcoin_bus::BusActor;
use starcoin_chain::BlockChain;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_storage::Storage;
use std::sync::Arc;
use txpool::TxPool;

pub fn start_txpool() -> (TxPool, Arc<Storage>, Arc<NodeConfig>) {
    let node_config = Arc::new(NodeConfig::random_for_test());

    let (storage, startup_info, _) =
        Genesis::init_storage(node_config.as_ref()).expect("init storage by genesis fail.");

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
    let (storage, startup_info, _) =
        Genesis::init_storage(config.as_ref()).expect("init storage by genesis fail.");

    let block_chain = BlockChain::new(config, *startup_info.get_master(), storage, None)?;
    Ok(block_chain)
}
