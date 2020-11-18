mod test_sync;

use config::{NodeConfig, SyncMode};
use futures::executor::block_on;
use logger::prelude::*;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use test_helper::run_node_by_config;
use traits::ChainAsyncService;

#[stest::test(timeout = 120)]
fn test_full_sync() {
    test_sync::test_sync(SyncMode::FULL)
}

//TODO fixme
#[ignore]
#[stest::test(timeout = 120)]
fn test_broadcast_with_difficulty() {
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
    sleep(Duration::from_millis(500));
    let block_1 = block_on(async { first_chain.clone().main_head_block().await.unwrap() });
    let number_1 = block_1.header().number();

    let mut second_config = NodeConfig::random_for_test();
    info!(
        "second peer : {:?}",
        second_config.network.self_peer_id().unwrap()
    );
    second_config.network.seeds = vec![first_config.network.self_address().unwrap()];
    //second_config.miner.enable_miner_client = false;
    second_config.sync.set_mode(SyncMode::FULL);

    let second_node = run_node_by_config(Arc::new(second_config)).unwrap();
    let second_chain = second_node.chain_service().unwrap();

    let not_broadcast_block = second_node.generate_block().unwrap();
    let mut number_2 = 0;
    for i in 0..10 as usize {
        std::thread::sleep(Duration::from_secs(2));
        let block_2 = block_on(async { second_chain.clone().main_head_block().await.unwrap() });
        number_2 = block_2.header().number();
        debug!("index : {}, second chain number is {}", i, number_2);
        if number_2 == number_1 {
            break;
        }
    }
    assert_eq!(number_1, number_2, "two node is not sync.");

    let block_not_exist = block_on(async {
        first_chain
            .get_header_by_hash(&not_broadcast_block.header().id())
            .await
            .unwrap()
    });
    assert!(block_not_exist.is_none());

    let broadcast_block = second_node.generate_block().unwrap();

    sleep(Duration::from_millis(500));
    let block_must_exist = block_on(async {
        first_chain
            .get_header_by_hash(&broadcast_block.header().id())
            .await
            .unwrap()
    });
    assert!(block_must_exist.is_some());
    assert_eq!(
        block_must_exist.unwrap().id(),
        broadcast_block.header().id()
    );
}
