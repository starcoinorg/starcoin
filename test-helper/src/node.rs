// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::executor::block_on;
use starcoin_config::NodeConfig;
use starcoin_node::node::NodeService;
use starcoin_node::NodeHandle;
use starcoin_node_api::node_service::NodeAsyncService;
use std::sync::Arc;

pub fn run_test_node() -> Result<NodeHandle> {
    let config = NodeConfig::random_for_test();
    run_node_by_config(Arc::new(config))
}

pub fn run_node_by_config(config: Arc<NodeConfig>) -> Result<NodeHandle> {
    let logger_handle = starcoin_logger::init_for_test();
    let node_handle = NodeService::launch(config, logger_handle)?;
    block_on(async { node_handle.node_service().stop_pacemaker().await })?;
    Ok(node_handle)
}
