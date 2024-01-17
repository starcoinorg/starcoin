// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Ok, Result};
use futures::executor::block_on;
use network_api::{PeerId, PeerProvider, PeerSelector, PeerStrategy};
use starcoin_config::*;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_node::NodeHandle;
use starcoin_sync::verified_rpc_client::VerifiedRpcClient;
use starcoin_types::block::BlockHeader;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct DagBlockInfo {
    pub header: BlockHeader,
    pub children: Vec<HashValue>,
}

#[stest::test]
fn test_verified_client_for_dag() {
    starcoin_types::block::set_test_flexidag_fork_height(10);
    let (local_handle, target_handle, target_peer_id) =
        init_two_node().expect("failed to initalize the local and target node");

    let network = local_handle.network();
    // PeerProvider
    let peer_info = block_on(network.get_peer(target_peer_id))
        .expect("failed to get peer info")
        .expect("failed to peer info for it is none");
    let peer_selector = PeerSelector::new(vec![peer_info], PeerStrategy::default(), None);
    let rpc_client = VerifiedRpcClient::new(peer_selector, network);
    // testing dag rpc
    let target_dag_blocks =
        generate_dag_block(&target_handle, 5).expect("failed to generate dag block");
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

fn init_two_node() -> Result<(NodeHandle, NodeHandle, PeerId)> {
    // network1 initialization
    let (local_handle, local_net_addr) = {
        let local_config = NodeConfig::random_for_test();
        let net_addr = local_config.network.self_address();
        debug!("Local node address: {:?}", net_addr);
        (gen_chain_env(local_config).unwrap(), net_addr)
    };

    // network2 initialization
    let (target_handle, target_peer_id) = {
        let mut target_config = NodeConfig::random_for_test();
        target_config.network.seeds = vec![local_net_addr].into();
        let target_peer_id = target_config.network.self_peer_id();
        (gen_chain_env(target_config).unwrap(), target_peer_id)
    };
    Ok((local_handle, target_handle, target_peer_id))
}

fn generate_dag_block(handle: &NodeHandle, count: usize) -> Result<Vec<DagBlockInfo>> {
    let mut result = vec![];
    let dag = handle.get_dag()?;
    while result.len() < count {
        let block = handle.generate_block()?;
        if block.header().is_dag() {
            result.push(block);
        }
    }
    Ok(result
        .into_iter()
        .map(|block| DagBlockInfo {
            header: block.header().clone(),
            children: dag.get_children(block.header().id()).unwrap(),
        })
        .collect::<Vec<DagBlockInfo>>())
}

fn gen_chain_env(config: NodeConfig) -> Result<NodeHandle> {
    test_helper::run_node_by_config(Arc::new(config))
}
