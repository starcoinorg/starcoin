// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::clock::Duration;
use starcoin_config::NodeConfig;
use starcoin_node::run_dev_node;
use std::sync::Arc;
use std::thread;

#[test]
fn test_run_node() {
    starcoin_logger::init_for_test();
    let config = Arc::new(NodeConfig::random_for_test());
    let handle = run_dev_node(config);
    thread::sleep(Duration::from_secs(10));
    handle.stop().unwrap()
}
