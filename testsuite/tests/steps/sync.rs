// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use starcoin_config::{ChainNetwork, NodeConfig, StarcoinOpt, SyncMode};
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
            opt.seed = Some(
                "/ip4/127.0.0.1/tcp/59753/p2p/12D3KooWMLGtRBKR31BpSdAxNHe8Qwv2rGiQUJpVuFFFoNTejq79"
                    .parse()
                    .unwrap(),
            );
            let config = NodeConfig::load_with_opt(&opt).unwrap();
            world.node_config = Some(config)
        })
        .given("a rpc client", |world: &mut MyWorld, _step| {
            let path = world.ipc_path.as_ref().take().unwrap();
            let client = RpcClient::connect_ipc(path).unwrap();
            info!("rpc client created!");
            world.rpc_client = Some(client)
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
        })
        .then("node stop", |world: &mut MyWorld, _step| {
            thread::sleep(Duration::from_secs(5));
            let handle = world.node_handle.take().unwrap();
            let result = handle.stop();
            assert!(result.is_ok());
        });
    builder.build()
}
