// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::executor::block_on;
use network_api::PeerProvider;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// test peer shutdown and reconnect
#[stest::test]
fn test_reconnected_peers() -> anyhow::Result<()> {
    let node_config1 = Arc::new(NodeConfig::random_for_test());
    let node1 = test_helper::run_node_by_config(node_config1.clone())?;

    let node1_network = node1.network();

    let peers = block_on(async { node1_network.peer_set().await })?;
    assert_eq!(peers.len(), 0);

    let mut node_config2 = NodeConfig::random_for_test();
    node_config2.network.seeds = vec![node_config1.network.self_address()].into();
    let node_config2 = Arc::new(node_config2);
    let node2 = test_helper::run_node_by_config(node_config2.clone())?;

    thread::sleep(Duration::from_secs(2));

    let network_state = block_on(async { node1_network.network_state().await })?;
    assert_eq!(network_state.connected_peers.len(), 1);

    let peers = block_on(async { node1_network.peer_set().await })?;
    assert_eq!(peers.len(), 1);

    // stop node2, node1's peers is empty
    node2.stop()?;
    thread::sleep(Duration::from_secs(3));
    loop {
        let network_state = block_on(async { node1_network.network_state().await })?;
        debug!("network_state: {:?}", network_state);
        if network_state.connected_peers.is_empty() {
            break;
        }
        thread::sleep(Duration::from_secs(1));
        //assert_eq!(network_state.connected_peers.len(), 0);
    }

    let peers = block_on(async { node1_network.peer_set().await })?;
    assert_eq!(peers.len(), 0);

    //start node2 again.
    let node2 = test_helper::run_node_by_config(node_config2)?;
    thread::sleep(Duration::from_secs(2));

    let network_state = block_on(async { node1_network.network_state().await })?;
    assert_eq!(network_state.connected_peers.len(), 1);

    let peers = block_on(async { node1_network.peer_set().await })?;
    assert_eq!(peers.len(), 1);
    node2.stop()?;
    node1.stop()?;
    Ok(())
}
