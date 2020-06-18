// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::clock::Duration;
use starcoin_config::NodeConfig;
use starcoin_node::run_dev_node;
use std::sync::Arc;
use std::thread;

#[stest::test]
fn test_run_node() {
    let node_config = NodeConfig::random_for_test();
    node_config.network.disable_seed = true;
    let config = Arc::new(node_config);
    let handle = run_dev_node(config);
    thread::sleep(Duration::from_secs(5));
    handle.stop().unwrap()
}
