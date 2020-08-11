use crate::TxPool;
use starcoin_bus::BusActor;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use std::sync::Arc;
use storage::Storage;

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
