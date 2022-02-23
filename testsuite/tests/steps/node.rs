// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use starcoin_config::{ChainNetworkID, NodeConfig, StarcoinOpt};
use starcoin_logger::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .given("a test node config", |world: &mut MyWorld, _step| {
            let mut opt = StarcoinOpt::default();
            opt.net = Some(ChainNetworkID::TEST);
            opt.metrics.disable_metrics = Some(false);
            let config = NodeConfig::load_with_opt(&opt).unwrap();
            info!("config: {:?}", config);
            world.node_config = Some(config)
        })
        .given("a dev node config", |world: &mut MyWorld, _step| {
            let mut opt = StarcoinOpt::default();
            opt.net = Some(ChainNetworkID::DEV);
            opt.metrics.disable_metrics = Some(false);
            let config = NodeConfig::load_with_opt(&opt).unwrap();
            world.node_config = Some(config)
        })
        .given("halley node config", |world: &mut MyWorld, _step| {
            let mut opt = StarcoinOpt::default();
            opt.net = Some(ChainNetworkID::HALLEY);
            opt.metrics.disable_metrics = Some(false);
            opt.base_data_dir = Some(PathBuf::from(starcoin_config::temp_dir().as_ref()));
            let config = NodeConfig::load_with_opt(&opt).unwrap();
            world.node_config = Some(config)
        })
        .given("node handle", |world: &mut MyWorld, _step| {
            let node_config = world.node_config.as_ref().take().unwrap();
            let handle = starcoin_node::run_node(Arc::new(node_config.clone()))
                .unwrap_or_else(|e| panic!("run node fail:{:?}", e));
            world.node_handle = Some(handle)
        })
        .then("stop", |world: &mut MyWorld, _step| {
            info!("try to stop world.");
            //drop client first.
            if world.default_rpc_client.is_some() {
                let client = world.default_rpc_client.take().unwrap();
                //arc client should no more reference at stop step.
                let client = Arc::try_unwrap(client).ok().unwrap();
                client.close();
                info!("default rpc client stopped.");
            }
            if world.rpc_client2.is_some() {
                let client = world.rpc_client2.take().unwrap();
                let client = Arc::try_unwrap(client).ok().unwrap();
                client.close();
                info!("rpc_client2 stopped.");
            }
            // stop node
            if world.node_handle.is_some() {
                let node_handle = world.node_handle.take().unwrap();
                if let Err(e) = node_handle.stop() {
                    error!("Node stop error: {:?}", e)
                }
                info!("node stopped.");
            }
        })
        .then("get node info", |world: &mut MyWorld, _step| {
            let client = world.default_rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info();
            info!("node_info: {:?}", node_info);
        })
        .then("get node status", |world: &mut MyWorld, _step| {
            let client = world.default_rpc_client.as_ref().take().unwrap();
            let status = client.clone().node_status();
            assert!(status.is_ok());
            assert_eq!(status.unwrap(), true);
        })
        .then("get node peers", |world: &mut MyWorld, _step| {
            let client = world.default_rpc_client.as_ref().take().unwrap();
            let peers = client.clone().node_peers();
            info!("peers: {:?}", peers);
        });
    builder.build()
}
