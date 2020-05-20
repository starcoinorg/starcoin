// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use cucumber::{after, before, cucumber, Steps, StepsBuilder};
use starcoin_config::{ChainConfig, ChainNetwork, Connect, NodeConfig, StarcoinOpt};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_node::NodeHandle;
use starcoin_rpc_client::RpcClient;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{BlockStore, Storage};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Default)]
pub struct MyWorld {
    node_config: Option<NodeConfig>,
    storage: Option<Storage>,
    node_handle: Option<NodeHandle>,
    rpc_client: Option<RpcClient>,
}
impl MyWorld {
    pub fn storage(&self) -> Option<&Storage> {
        match &self.storage {
            Some(storage) => Some(storage),
            _ => None,
        }
    }
}

impl cucumber::World for MyWorld {}

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .given("a node config", |world: &mut MyWorld, _step| {
            let mut opt = StarcoinOpt::default();
            opt.net = Some(ChainNetwork::Dev);
            opt.data_dir = Some("./conf".parse().unwrap());
            let connect = Connect::IPC(Some("./conf/my.ipc".parse().unwrap()));
            opt.connect = Some(connect);
            let config = NodeConfig::load_with_opt(&opt).unwrap();
            world.node_config = Some(config)
        })
        .given("a storage", |world: &mut MyWorld, _step| {
            let cache_storage = Arc::new(CacheStorage::new());
            let db_storage = Arc::new(DBStorage::new(starcoin_config::temp_path().as_ref()));
            let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
                cache_storage,
                db_storage,
            ))
            .unwrap();
            world.storage = Some(storage)
        })
        .given("a rpc client", |world: &mut MyWorld, _step| {
            let node_config = world.node_config.as_ref().take().unwrap();
            let client = RpcClient::connect_ipc(node_config.clone().rpc.get_ipc_file()).unwrap();
            world.rpc_client = Some(client)
        })
        .given("a node handle", |world: &mut MyWorld, _step| {
            let node_config = world.node_config.as_ref().take().unwrap();
            let handle = starcoin_node::run_dev_node(Arc::new(node_config.clone()));
            world.node_handle = Some(handle)
        })
        .then("get node info", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info();
            assert!(node_info.is_ok());
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
            assert!(peers.is_ok());
        })
        .then("node handle stop", |world: &mut MyWorld, _step| {
            thread::sleep(Duration::from_secs(2));
            world.node_handle.as_ref().take().unwrap().stop().unwrap();
        });
    builder.build()
}

// Declares a before handler function named `a_before_fn`
before!(a_before_fn => |_scenario| {
});

// Declares an after handler function named `an_after_fn`
after!(an_after_fn => |_scenario| {
});

// A setup function to be called before everything else
fn setup() {}

mod steps;
use steps::*;

cucumber! {
    features: "./features", // Path to our feature files
    world: World, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        crate::steps, // the `steps!` macro creates a `steps` function in a module
        transaction::steps,
    ],
    setup: setup, // Optional; called once before everything
    before: &[
        a_before_fn // Optional; called before each scenario
    ],
    after: &[
        an_after_fn // Optional; called after each scenario
    ]
}
