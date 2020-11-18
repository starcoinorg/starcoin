use config::{NodeConfig, SyncMode};
use futures::executor::block_on;
use logger::prelude::*;
use std::thread::sleep;
use std::{sync::Arc, time::Duration};
use test_helper::run_node_by_config;
use traits::ChainAsyncService;

pub fn test_sync(sync_mode: SyncMode) {
    let first_config = Arc::new(NodeConfig::random_for_test());
    info!(
        "first peer : {:?}",
        first_config.network.self_peer_id().unwrap()
    );
    let first_node = run_node_by_config(first_config.clone()).unwrap();
    let first_chain = first_node.chain_service().unwrap();
    let count = 5;
    for _i in 0..count {
        first_node.generate_block().unwrap();
    }
    //wait block generate.
    sleep(Duration::from_millis(500));
    let block_1 = block_on(async { first_chain.main_head_block().await.unwrap() });
    let number_1 = block_1.header().number();
    debug!("first chain head block number is {}", number_1);
    assert_eq!(number_1, count);

    let mut second_config = NodeConfig::random_for_test();
    info!(
        "second peer : {:?}",
        second_config.network.self_peer_id().unwrap()
    );
    second_config.network.seeds = vec![first_config.network.self_address().unwrap()];
    second_config.miner.enable_miner_client = false;
    second_config.sync.set_mode(sync_mode);

    let second_node = run_node_by_config(Arc::new(second_config)).unwrap();
    let second_chain = second_node.chain_service().unwrap();

    //TODO add more check.
    let mut number_2 = 0;
    for i in 0..10 as usize {
        std::thread::sleep(Duration::from_secs(2));
        let block_2 = block_on(async { second_chain.main_head_block().await.unwrap() });
        number_2 = block_2.header().number();
        debug!("index : {}, second chain number is {}", i, number_2);
        if number_2 == number_1 {
            break;
        }
    }
    assert_eq!(number_1, number_2, "two node is not sync.");
    second_node.stop().unwrap();
    first_node.stop().unwrap();
}
