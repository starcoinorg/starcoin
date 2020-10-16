// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::NodeConfig;
use futures::executor::block_on;
use logger::prelude::*;
use network_api::PeerProvider;
use starcoin_accumulator::tree_store::mock::MockAccumulatorStore;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_sync::task::BlockAccumulatorSyncTask;
use starcoin_sync::verified_rpc_client::VerifiedRpcClient;
use std::thread::sleep;
use std::{sync::Arc, time::Duration};
use test_helper::run_node_by_config;
use traits::ChainAsyncService;

#[ignore]
#[stest::test]
pub fn test_accumulator_sync() {
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
    let block_1 = block_on(async { first_chain.master_head_block().await.unwrap().unwrap() });
    let number_1 = block_1.header().number();
    debug!("first chain head block number is {}", number_1);
    assert_eq!(number_1, count);
    let block_info1 = block_on(async {
        first_chain
            .get_block_info_by_hash(&block_1.id())
            .await
            .unwrap()
            .unwrap()
    });

    let mut second_config = NodeConfig::random_for_test();
    info!(
        "second peer : {:?}",
        second_config.network.self_peer_id().unwrap()
    );
    second_config.network.seeds = vec![first_config.network.self_address().unwrap()];
    second_config.miner.enable_miner_client = false;

    let second_node = run_node_by_config(Arc::new(second_config)).unwrap();
    let genesis = second_node.genesis();
    let network = second_node.network();

    let store = Arc::new(MockAccumulatorStore::new());
    let accumulator = MerkleAccumulator::new_empty(store.clone());
    accumulator.append(&[genesis.block().id()]).unwrap();
    accumulator.flush().unwrap();

    let peer_selector = block_on(async { network.peer_selector().await }).unwrap();
    let client = VerifiedRpcClient::new(peer_selector, network);
    let sync_task = BlockAccumulatorSyncTask::new(
        store,
        accumulator.get_info(),
        block_info1.block_accumulator_info.clone(),
        client,
        2,
    );
    let accumulator_info = block_on(async { sync_task.await }).unwrap();
    assert_eq!(accumulator_info, block_info1.block_accumulator_info);

    second_node.stop().unwrap();
    first_node.stop().unwrap();
}
