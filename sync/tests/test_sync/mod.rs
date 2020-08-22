use config::{NodeConfig, SyncMode};
use futures_timer::Delay;
use logger::prelude::*;
use std::{sync::Arc, time::Duration};
use traits::ChainAsyncService;

pub async fn test_sync(sync_mode: SyncMode) {
    let first_config = Arc::new(NodeConfig::random_for_test());
    let first_node = starcoin_node::node::start(first_config.clone(), None)
        .await
        .unwrap();
    let first_chain = first_node.chain_actor.clone();
    let mut second_config = NodeConfig::random_for_test();
    second_config.network.seeds = vec![first_config.network.self_address().unwrap()];
    second_config.miner.enable_miner_client = false;
    second_config.sync.set_mode(sync_mode);

    let second_node = starcoin_node::node::start(Arc::new(second_config), None)
        .await
        .unwrap();
    let second_chain = second_node.chain_actor.clone();

    //TODO add more check.
    for i in 0..5 as usize {
        Delay::new(Duration::from_secs(1)).await;
        let block_1 = first_chain
            .clone()
            .master_head_block()
            .await
            .unwrap()
            .unwrap();
        let number_1 = block_1.header().number();
        debug!("index : {}, first chain number is {}", i, number_1);

        let block_2 = second_chain
            .clone()
            .master_head_block()
            .await
            .unwrap()
            .unwrap();
        let number_2 = block_2.header().number();
        debug!("index : {}, second chain number is {}", i, number_2);

        assert!(number_2 > 0);
    }
}
