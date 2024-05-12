mod common_test_sync_libs;
mod test_sync;

use anyhow::{Ok, Result};
use futures::executor::block_on;
use rand::random;
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainAsyncService, ChainReader, ExecutedBlock};
use starcoin_chain_service::ChainReaderService;
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_logger::prelude::*;
use starcoin_node::NodeHandle;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{ActorService, RegistryAsyncService, ServiceRef};
use starcoin_storage::Storage;
use starcoin_sync::sync::SyncService;
use starcoin_types::block::BlockNumber;
use starcoin_types::system_events::NewHeadBlock;
use starcoin_vm_types::on_chain_config::FlexiDagConfig;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use test_helper::{run_node_by_config, Account};

#[stest::test(timeout = 120)]
fn test_full_sync() {
    test_sync::test_sync()
}

//FIX ME
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
        .stop_service(SyncService::service_name().to_string())
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

#[stest::test(timeout = 120)]
fn test_sync_and_notification() {
    let first_config = Arc::new(NodeConfig::random_for_test());
    info!("first peer : {:?}", first_config.network.self_peer_id());
    let first_node = run_node_by_config(first_config.clone()).unwrap();
    for _i in 0..5 {
        first_node.generate_block().unwrap();
    }
    sleep(Duration::from_millis(500));

    let mut second_config = NodeConfig::random_for_test();
    info!("second peer : {:?}", second_config.network.self_peer_id());
    second_config.network.seeds = vec![first_config.network.self_address()].into();
    //second_config.miner.enable_miner_client = false;

    let second_node = run_node_by_config(Arc::new(second_config)).unwrap();
    //wait first sync.
    wait_two_node_synced(&first_node, &second_node);

    // generate block
    for _i in 0..10 {
        let r: u32 = random();
        if r % 2 == 0 {
            let _broadcast_block = first_node.generate_block().unwrap();
        } else {
            let _broadcast_block = second_node.generate_block().unwrap();
        }
    }
    // wait sync again.
    wait_two_node_synced(&first_node, &second_node);
}

fn wait_two_node_synced(first_node: &NodeHandle, second_node: &NodeHandle) {
    let first_chain = first_node.chain_service().unwrap();
    let second_chain = second_node.chain_service().unwrap();

    for i in 0..100 {
        let block_1 = block_on(async { first_chain.clone().main_head_block().await.unwrap() });
        let block_2 = block_on(async { second_chain.clone().main_head_block().await.unwrap() });
        debug!(
            "check sync index : {}, first chain number is:{}, second chain number is: {}",
            i,
            block_1.header().number(),
            block_2.header().number()
        );
        if block_1 == block_2 {
            break;
        } else if i == 100 {
            panic!(
                "two node is not synced, first: {:?}, second: {:?}",
                block_1.header(),
                block_2.header(),
            );
        } else {
            std::thread::sleep(Duration::from_millis(500));
        }
    }
}

async fn check_synced(
    target_hash: HashValue,
    chain_service: ServiceRef<ChainReaderService>,
) -> Result<bool> {
    loop {
        if target_hash
            == chain_service
                .main_head_block()
                .await
                .expect("failed to get main head block")
                .id()
        {
            debug!("succeed to sync main block id: {:?}", target_hash);
            break;
        } else {
            debug!("waiting for sync, now sleep 60 second");
            async_std::task::sleep(Duration::from_millis(500)).await;
        }
    }
    Ok(true)
}

/// Just for test
pub fn execute_dag_poll_block(node_handle: &NodeHandle, fork_number: BlockNumber) -> Result<u64> {
    let timestamp = block_on(async move {
        let registry = node_handle.registry();
        let node_config = registry
            .get_shared::<Arc<NodeConfig>>()
            .await
            .expect("Failed to get node config");
        let time_service = node_config.net().time_service();
        let chain_service = registry
            .service_ref::<ChainReaderService>()
            .await
            .expect("failed to get chain reader service");
        let header_hash = chain_service
            .main_head_header()
            .await
            .expect("failed to get header hash")
            .id();
        let storage = registry
            .get_shared::<Arc<Storage>>()
            .await
            .expect("failed to get storage");
        let dag = registry
            .get_shared::<BlockDAG>()
            .await
            .expect("failed to get dag");
        let mut chain = BlockChain::new(time_service, header_hash, storage, None, dag)
            .expect("failed to get new the chain");
        let net = node_config.net();
        let current_number = chain.status().head().number();
        chain = test_helper::dao::modify_on_chain_config_by_dao_block(
            Account::new(),
            chain,
            net,
            test_helper::dao::vote_flexi_dag_config(net, fork_number),
            test_helper::dao::on_chain_config_type_tag(FlexiDagConfig::type_tag()),
            test_helper::dao::execute_script_on_chain_config(net, FlexiDagConfig::type_tag(), 0u64),
        )
        .expect("failed to execute script for poll");

        let bus = registry
            .service_ref::<BusService>()
            .await
            .expect("failed to get bus service");
        // broadcast poll blocks
        for block_number in current_number + 1..=chain.status().head().number() {
            let block = chain
                .get_block_by_number(block_number)
                .expect("failed to get block by number")
                .unwrap();
            let block_info = chain
                .get_block_info(Some(block.id()))
                .expect("failed to get block info")
                .unwrap();
            bus.broadcast(NewHeadBlock {
                executed_block: Arc::new(ExecutedBlock::new(block, block_info)),
            })
            .expect("failed to broadcast new head block");
        }

        loop {
            if chain_service
                .main_head_block()
                .await
                .expect("failed to get main head block")
                .header()
                .number()
                == chain.status().head().number()
            {
                break;
            } else {
                async_std::task::sleep(Duration::from_millis(500)).await;
            }
        }
        chain.time_service().now_millis()
    });
    Ok(timestamp)
}

#[stest::test(timeout = 720)]
fn test_multiple_node_sync() {
    let nodes =
        common_test_sync_libs::init_multiple_node(5).expect("failed to initialize multiple nodes");

    // for node in &nodes {
    //     execute_dag_poll_block(node, 20).expect("execute poll block failed");
    // }

    let main_node = &nodes.first().expect("failed to get main node");

    common_test_sync_libs::generate_block(main_node, 20).expect("failed to generate dag block");

    let main_node_chain_service = main_node
        .chain_service()
        .expect("failed to get main node chain service");
    let chain_service_1 = nodes[1]
        .chain_service()
        .expect("failed to get the chain service");
    let chain_service_2 = nodes[2]
        .chain_service()
        .expect("failed to get the chain service");
    let chain_service_3 = nodes[3]
        .chain_service()
        .expect("failed to get the chain service");
    let chain_service_4 = nodes[4]
        .chain_service()
        .expect("failed to get the chain service");

    block_on(async move {
        let main_block = main_node_chain_service
            .main_head_block()
            .await
            .expect("failed to get main head block");

        nodes[1]
            .start_to_sync()
            .await
            .expect("failed to start to sync");
        nodes[2]
            .start_to_sync()
            .await
            .expect("failed to start to sync");
        nodes[3]
            .start_to_sync()
            .await
            .expect("failed to start to sync");
        nodes[4]
            .start_to_sync()
            .await
            .expect("failed to start to sync");

        check_synced(main_block.id(), chain_service_1)
            .await
            .expect("failed to check sync");
        check_synced(main_block.id(), chain_service_2)
            .await
            .expect("failed to check sync");
        check_synced(main_block.id(), chain_service_3)
            .await
            .expect("failed to check sync");
        check_synced(main_block.id(), chain_service_4)
            .await
            .expect("failed to check sync");

        // close
        nodes.into_iter().for_each(|handle| {
            handle
                .stop()
                .expect("failed to shutdown the node normally!");
        });
    });
}
