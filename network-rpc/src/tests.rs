// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::executor::block_on;
use starcoin_config::*;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::{gen_client as starcoin_gen_client, GetBlockIds, Ping};
use starcoin_node::NodeHandle;
use std::sync::Arc;

#[stest::test]
fn test_network_rpc() {
    let (handle1, net_addr_1) = {
        let config_1 = NodeConfig::random_for_test();
        let net_addr = config_1.network.self_address();
        debug!("First node address: {:?}", net_addr);
        (gen_chain_env(config_1).unwrap(), net_addr)
    };

    let network_1 = handle1.network();
    let (handle2, peer_id_2) = {
        let mut config_2 = NodeConfig::random_for_test();
        config_2.network.seeds = vec![net_addr_1].into();
        let peer_id_2 = config_2.network.self_peer_id();
        (gen_chain_env(config_2).unwrap(), peer_id_2)
    };
    handle2.generate_block().unwrap();

    // network rpc client for chain 1
    let client = starcoin_gen_client::NetworkRpcClient::new(network_1);

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

    let rpc_info = starcoin_gen_client::get_rpc_info();
    debug!("{:?}", rpc_info);

    let req = GetBlockIds {
        start_number: 0,
        reverse: false,
        max_size: 100,
    };
    let block_ids = block_on(async { client.get_block_ids(peer_id_2.clone(), req).await.unwrap() });
    assert_eq!(2, block_ids.len());

    let blocks = block_on(async {
        client
            .get_blocks(peer_id_2.clone(), block_ids)
            .await
            .unwrap()
    });
    assert_eq!(2, blocks.len());

    handle2.stop().unwrap();
    handle1.stop().unwrap();
}

fn gen_chain_env(config: NodeConfig) -> Result<NodeHandle> {
    test_helper::run_node_by_config(Arc::new(config))
}
