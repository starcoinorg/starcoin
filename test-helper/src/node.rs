// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_node::NodeHandle;
use std::sync::Arc;

pub fn run_test_node() -> Result<NodeHandle> {
    let config = NodeConfig::random_for_test();
    let logger_handle = starcoin_logger::init_for_test();
    starcoin_node::run_node_with_log(Arc::new(config), logger_handle)
}

pub fn run_node_by_config(config: Arc<NodeConfig>) -> Result<NodeHandle> {
    let logger_handle = starcoin_logger::init_for_test();
    starcoin_node::run_node_with_log(config, logger_handle)
}
