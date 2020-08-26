use config::{NodeConfig, SyncMode};
use futures::executor::block_on;
use logger::prelude::*;
use std::{sync::Arc, time::Duration};
use test_helper::run_node_by_config;
use traits::ChainAsyncService;

pub fn test_sync(sync_mode: SyncMode) {
    let first_config = Arc::new(NodeConfig::random_for_test());
    let first_node = run_node_by_config(first_config.clone()).unwrap();
    let first_chain = first_node.start_handle().chain_actor.clone();
    let mut second_config = NodeConfig::random_for_test();
    second_config.network.seeds = vec![first_config.network.self_address().unwrap()];
    second_config.miner.enable_miner_client = false;
    second_config.sync.set_mode(sync_mode);

    let second_node = run_node_by_config(Arc::new(second_config)).unwrap();
    let second_chain = second_node.start_handle().chain_actor.clone();

    //TODO add more check.
    for i in 0..5 as usize {
        std::thread::sleep(Duration::from_secs(2));
        let block_1 = block_on(async {
            first_chain
                .clone()
                .master_head_block()
                .await
                .unwrap()
                .unwrap()
        });
        let number_1 = block_1.header().number();
        debug!("index : {}, first chain number is {}", i, number_1);

        let block_2 = block_on(async {
            second_chain
                .clone()
                .master_head_block()
                .await
                .unwrap()
                .unwrap()
        });
        let number_2 = block_2.header().number();
        debug!("index : {}, second chain number is {}", i, number_2);

        assert!(number_2 > 0);
    }
    second_node.stop().unwrap();
    first_node.stop().unwrap();
}
