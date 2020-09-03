// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use config::*;
use futures::executor::block_on;
use logger::prelude::*;
use network_api::NetworkService;
use starcoin_network_rpc_api::{gen_client, GetBlockHeadersByNumber, GetStateWithProof};
use starcoin_node::NodeHandle;
use state_api::StateWithProof;
use std::sync::Arc;
use types::{access_path, account_config::genesis_address, block::BlockHeader};
use vm_types::move_resource::MoveResource;
use vm_types::on_chain_config::EpochResource;

#[stest::test]
fn test_network_rpc() {
    let (handle1, net_addr_1) = {
        let config_1 = NodeConfig::random_for_test();
        let net_addr = config_1.network.self_address().unwrap();
        debug!("First node address: {:?}", net_addr);
        (gen_chain_env(config_1).unwrap(), net_addr)
    };

    let network_1 = handle1.start_handle().network.clone();
    let handle2 = {
        let mut config_2 = NodeConfig::random_for_test();
        config_2.network.seeds = vec![net_addr_1];
        gen_chain_env(config_2).unwrap()
    };
    handle2.generate_block().unwrap();

    let network_2 = handle2.start_handle().network.clone();
    // network rpc client for chain 1
    let peer_id_2 = network_2.identify().clone();
    let client = gen_client::NetworkRpcClient::new(network_1);

    let access_path =
        access_path::AccessPath::new(genesis_address(), EpochResource::resource_path());

    let req = GetBlockHeadersByNumber::new(1, 1, 1);
    let resp: Vec<BlockHeader> = block_on(async {
        client
            .get_headers_by_number(peer_id_2.clone().into(), req)
            .await
            .unwrap()
    });
    assert!(!resp.is_empty());
    let state_root = resp[0].state_root;

    let state_req = GetStateWithProof {
        state_root,
        access_path: access_path.clone(),
    };
    let state_with_proof: StateWithProof = block_on(async {
        client
            .get_state_with_proof(peer_id_2.clone().into(), state_req)
            .await
            .unwrap()
    });
    let state = state_with_proof.state.unwrap();
    let epoch = scs::from_bytes::<EpochResource>(state.as_slice()).unwrap();
    state_with_proof
        .proof
        .verify(state_root, access_path, Some(&state))
        .unwrap();
    debug!("{:?}", epoch);

    let rpc_info = gen_client::get_rpc_info();
    debug!("{:?}", rpc_info);

    let ping = block_on(async {
        client
            .ping(peer_id_2.clone().into(), "hello".to_string())
            .await
    });
    match ping {
        Err(e) => debug!("{}", e),
        Ok(_) => panic!(""),
    }
    handle2.stop().unwrap();
    handle1.stop().unwrap();
}

fn gen_chain_env(config: NodeConfig) -> Result<NodeHandle> {
    test_helper::run_node_by_config(Arc::new(config))
}
