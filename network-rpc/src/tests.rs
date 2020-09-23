// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use config::*;
use futures::executor::block_on;
use network_api::PeerProvider;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::{
    gen_client as starcoin_gen_client, GetBlockHeadersByNumber, GetStateWithProof, Ping,
};
use starcoin_node::NodeHandle;
use starcoin_state_api::StateWithProof;
use starcoin_types::{access_path, account_config::genesis_address, block::BlockHeader};
use std::sync::Arc;
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
    let peer_id_2 = network_2.identify();
    let client = starcoin_gen_client::NetworkRpcClient::new(network_1);

    let access_path =
        access_path::AccessPath::new(genesis_address(), EpochResource::resource_path());

    //ping ok
    let req = Ping {
        msg: "ping_test".to_string(),
        err: false,
    };
    let resp: String =
        block_on(async { client.ping(peer_id_2.clone(), req.clone()).await.unwrap() });
    assert_eq!(req.msg, resp);

    //ping err
    let ping = block_on(async {
        client
            .ping(
                peer_id_2.clone(),
                Ping {
                    msg: "ping_test".to_string(),
                    err: true,
                },
            )
            .await
    });
    assert!(ping.is_err(), "expect return err, but return ok");

    let req = GetBlockHeadersByNumber::new(1, 1, 1);
    let resp: Vec<BlockHeader> = block_on(async {
        client
            .get_headers_by_number(peer_id_2.clone(), req)
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
            .get_state_with_proof(peer_id_2.clone(), state_req)
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

    let rpc_info = starcoin_gen_client::get_rpc_info();
    debug!("{:?}", rpc_info);

    handle2.stop().unwrap();
    handle1.stop().unwrap();
}

fn gen_chain_env(config: NodeConfig) -> Result<NodeHandle> {
    test_helper::run_node_by_config(Arc::new(config))
}
