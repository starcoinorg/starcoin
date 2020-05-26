// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use starcoin_config::{ChainNetwork, NodeConfig, StarcoinOpt, SyncMode};
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .given("a node config", |world: &mut MyWorld, _step| {
            let mut opt = StarcoinOpt::default();
            opt.net = Some(ChainNetwork::Dev);
            opt.data_dir = Some("./dev".parse().unwrap());
            opt.sync_mode = SyncMode::FULL;
            opt.seed = Some(env!("STARCOIN_SEED").to_string().parse().unwrap());
            let config = NodeConfig::load_with_opt(&opt).unwrap();
            world.node_config = Some(config)
        })
        .given("local rpc client", |world: &mut MyWorld, _step| {
            let node_config = world.node_config.as_ref().take().unwrap();
            let client = RpcClient::connect_ipc(node_config.clone().rpc.get_ipc_file()).unwrap();
            info!("node local rpc client created!");
            world.local_rpc_client = Some(client)
        })
        .given("node handle", |world: &mut MyWorld, _step| {
            let node_config = world.node_config.as_ref().take().unwrap();
            let handle = starcoin_node::run_dev_node(Arc::new(node_config.clone()));
            world.node_handle = Some(handle)
        })
        .then("basic check", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let status = client.clone().node_status();
            assert!(status.is_ok());
            //read head from remote
            let remote_chain = client.clone().chain_head().unwrap();
            let header = remote_chain.get_head();
            let local_client = world.local_rpc_client.as_ref().take().unwrap();
            let local_chain = local_client.clone().chain_head().unwrap();
            let local_header = local_chain.get_head();
            assert_eq!(header.clone(), local_header.clone());
        })
        .then("node stop", |world: &mut MyWorld, _step| {
            thread::sleep(Duration::from_secs(5));
            let handle = world.node_handle.take().unwrap();
            let result = handle.stop();
            assert!(result.is_ok());
        });
    builder.build()
}
