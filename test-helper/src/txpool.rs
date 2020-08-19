// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_bus::BusActor;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_storage::Storage;
use starcoin_txpool::TxPool;
use std::sync::Arc;

pub fn start_txpool() -> (TxPool, Arc<Storage>, Arc<NodeConfig>) {
    let node_config = Arc::new(NodeConfig::random_for_test());

    let (storage, startup_info, _) =
        Genesis::init_storage_for_test(node_config.net()).expect("init storage by genesis fail.");

    let bus = BusActor::launch();

    let pool = TxPool::start(
        node_config.clone(),
        storage.clone(),
        *startup_info.get_master(),
        bus,
    );
    (pool, storage, node_config)
}
