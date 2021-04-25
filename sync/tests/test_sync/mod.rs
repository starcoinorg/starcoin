use config::NodeConfig;
use futures::executor::block_on;
use logger::prelude::*;
use starcoin_chain_service::ChainAsyncService;
use starcoin_sync_api::SyncAsyncService;
use std::thread::sleep;
use std::{sync::Arc, time::Duration};
use test_helper::run_node_by_config;

pub fn test_sync() {
    let first_config = Arc::new(NodeConfig::random_for_test());
    info!("first peer : {:?}", first_config.network.self_peer_id());
    let first_node = run_node_by_config(first_config.clone()).unwrap();
    let first_chain = first_node.chain_service().unwrap();
    let first_network = first_node.network();
    let first_peer_id = first_config.network.self_peer_id();
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
    info!("second peer : {:?}", second_config.network.self_peer_id());
    second_config.network.seeds = vec![first_config.network.self_address()].into();
    second_config.miner.disable_miner_client = Some(true);

    let second_node = run_node_by_config(Arc::new(second_config.clone())).unwrap();
    let second_chain = second_node.chain_service().unwrap();
    let second_sync_service = second_node.sync_service().unwrap();
    let second_network = second_node.network();
    let second_peer_id = second_config.network.self_peer_id();
    std::thread::sleep(Duration::from_secs(2));
    block_on(async {
        assert!(
            second_network.is_connected(first_peer_id).await,
            "second node should connect to first node."
        );
        assert!(
            first_network.is_connected(second_peer_id).await,
            "first node should connect to second node."
        );
    });
    //Try to trigger sync.
    block_on(async {
        second_sync_service
            .start(false, vec![], false, None)
            .await
            .unwrap();
    });
    //TODO add more check.
    let mut number_2 = 0;
    for i in 0..10_usize {
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
