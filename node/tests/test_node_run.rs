// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::executor::block_on;
use starcoin_chain_service::ChainAsyncService;
use starcoin_config::NodeConfig;
use starcoin_node::run_node;
use starcoin_node_api::node_service::NodeAsyncService;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[stest::test]
fn test_run_node() {
    let mut node_config = NodeConfig::random_for_test();
    node_config.network.disable_seed = true;
    let config = Arc::new(node_config);
    let handle = run_node(config).unwrap();
    let services = handle.list_service().unwrap();
    println!("{:?}", services);
    thread::sleep(Duration::from_secs(5));
    handle.stop().unwrap()
}

#[stest::test]
fn test_generate_block() {
    let mut node_config = NodeConfig::random_for_test();
    node_config.network.disable_seed = true;
    let config = Arc::new(node_config);
    let handle = run_node(config).unwrap();
    let node_service = handle.node_service();
    let chain_service = handle.chain_service().unwrap();
    block_on(async { node_service.stop_pacemaker().await }).unwrap();
    let latest_block = block_on(async { chain_service.main_head_block().await }).unwrap();
    let count = 5;
    (0..count).for_each(|_| {
        handle.generate_block().unwrap();
    });
    thread::sleep(Duration::from_secs(1));
    let latest_block2 = block_on(async { chain_service.main_head_block().await }).unwrap();
    assert_eq!(
        latest_block.header().number() + count,
        latest_block2.header().number()
    );
    handle.stop().unwrap()
}
