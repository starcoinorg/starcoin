// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use starcoin_config::{ChainNetwork, NodeConfig, StarcoinOpt};
use std::path::PathBuf;
use std::sync::Arc;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .given("a dev node config", |world: &mut MyWorld, _step| {
            let mut opt = StarcoinOpt::default();
            opt.net = Some(ChainNetwork::Dev);
            opt.disable_metrics = true;
            opt.data_dir = Some(PathBuf::from(starcoin_config::temp_path().as_ref()));
            let mut config = NodeConfig::load_with_opt(&opt).unwrap();
            config.network.disable_seed = true;
            world.node_config = Some(config)
        })
        .given("halley node config", |world: &mut MyWorld, _step| {
            let mut opt = StarcoinOpt::default();
            opt.net = Some(ChainNetwork::Halley);
            opt.disable_metrics = true;
            opt.data_dir = Some(PathBuf::from(starcoin_config::temp_path().as_ref()));
            let config = NodeConfig::load_with_opt(&opt).unwrap();
            world.node_config = Some(config)
        })
        .given("node dev handle", |world: &mut MyWorld, _step| {
            let node_config = world.node_config.as_ref().take().unwrap();
            let handle = starcoin_node::run_node(Arc::new(node_config.clone()));
            world.node_handle = Some(handle)
        })
        .given("node handle", |world: &mut MyWorld, _step| {
            let node_config = world.node_config.as_ref().take().unwrap();
            let handle = starcoin_node::run_node(Arc::new(node_config.clone()));
            world.node_handle = Some(handle)
        })
        .then("node handle stop", |world: &mut MyWorld, _step| {
            let node_handle = world.node_handle.take().unwrap();
            let result = node_handle.stop();
            assert!(result.is_ok());
        })
        .then("get node info", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info();
//             assert!(node_info.is_ok());
        })
        .then("get node status", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let status = client.clone().node_status();
            assert!(status.is_ok());
            assert_eq!(status.unwrap(), true);
        })
        .then("get node peers", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let peers = client.clone().node_peers();
//             assert!(peers.is_ok());
        });
    builder.build()
}
