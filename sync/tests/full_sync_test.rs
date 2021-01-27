mod test_sync;

use config::NodeConfig;
use futures::executor::block_on;
use logger::prelude::*;
use starcoin_service_registry::ActorService;
use starcoin_sync::sync2::SyncService2;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use test_helper::run_node_by_config;
use traits::ChainAsyncService;

#[stest::test(timeout = 120)]
fn test_full_sync() {
    test_sync::test_sync()
}

#[ignore]
#[stest::test(timeout = 120)]
fn test_sync_by_notification() {
    let first_config = Arc::new(NodeConfig::random_for_test());
    info!("first peer : {:?}", first_config.network.self_peer_id());
    let first_node = run_node_by_config(first_config.clone()).unwrap();
    let first_chain = first_node.chain_service().unwrap();

    //wait node start
    sleep(Duration::from_millis(1000));

    let mut second_config = NodeConfig::random_for_test();
    info!("second peer : {:?}", second_config.network.self_peer_id());
    second_config.network.seeds = vec![first_config.network.self_address()].into();
    second_config.miner.disable_miner_client = Some(false);

    let second_node = run_node_by_config(Arc::new(second_config)).unwrap();
    // stop sync service and just use notification message to sync.
    second_node
        .stop_service(SyncService2::service_name().to_string())
        .unwrap();

    let second_chain = second_node.chain_service().unwrap();

    //wait node start and sync service stop.
    sleep(Duration::from_millis(1000));

    let count = 5;
    for _i in 0..count {
        first_node.generate_block().unwrap();
    }

    //wait block generate.
    sleep(Duration::from_millis(500));
    let block_1 = block_on(async { first_chain.main_head_block().await.unwrap() });
    let number_1 = block_1.header().number();

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

//TODO fixme
#[ignore]
#[stest::test(timeout = 120)]
fn test_broadcast_with_difficulty() {
    let first_config = Arc::new(NodeConfig::random_for_test());
    info!("first peer : {:?}", first_config.network.self_peer_id());
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
    info!("second peer : {:?}", second_config.network.self_peer_id());
    second_config.network.seeds = vec![first_config.network.self_address()].into();
    //second_config.miner.enable_miner_client = false;

    let second_node = run_node_by_config(Arc::new(second_config)).unwrap();
    let second_chain = second_node.chain_service().unwrap();

    let not_broadcast_block = second_node.generate_block().unwrap();
    let mut number_2 = 0;
    for i in 0..10_usize {
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
