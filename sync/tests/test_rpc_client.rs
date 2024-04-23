// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod common_test_sync_libs;

use futures::executor::block_on;
use network_api::{PeerProvider, PeerSelector, PeerStrategy};
use starcoin_logger::prelude::*;
use starcoin_sync::verified_rpc_client::VerifiedRpcClient;
use starcoin_types::block::BlockHeader;
use std::sync::Arc;

#[stest::test]
fn test_verified_client_for_dag() {
    starcoin_types::block::set_test_flexidag_fork_height(10);
    let (local_handle, target_handle, target_peer_id) = common_test_sync_libs::init_two_node()
        .expect("failed to initalize the local and target node");

    let network = local_handle.network();
    // PeerProvider
    let peer_info = block_on(network.get_peer(target_peer_id))
        .expect("failed to get peer info")
        .expect("failed to peer info for it is none");
    let peer_selector = PeerSelector::new(vec![peer_info], PeerStrategy::default(), None);
    let rpc_client = VerifiedRpcClient::new(peer_selector, network);
    // testing dag rpc
    let target_dag_blocks = common_test_sync_libs::generate_dag_block(&target_handle, 5)
        .expect("failed to generate dag block");
    target_dag_blocks.into_iter().for_each(|target_dag_block| {
        let dag_children_from_client_rpc =
            block_on(rpc_client.get_dag_block_children(vec![target_dag_block.header.id()]))
                .expect("failed to get dag block children");
        info!(
            "get dag children for:{},{:?}",
            target_dag_block.header.id(),
            dag_children_from_client_rpc
        );
        assert!(target_dag_block
            .clone()
            .children
            .into_iter()
            .all(|child| { dag_children_from_client_rpc.contains(&child) }));

        assert!(dag_children_from_client_rpc
            .into_iter()
            .all(|child| { target_dag_block.children.contains(&child) }));
    });
    starcoin_types::block::reset_test_custom_fork_height();
    target_handle.stop().unwrap();
    local_handle.stop().unwrap();
}
