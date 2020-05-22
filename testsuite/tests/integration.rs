// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use cucumber::{after, before, cucumber, Steps, StepsBuilder};
use starcoin_config::{ChainConfig, ChainNetwork, Connect, NodeConfig, StarcoinOpt};
use starcoin_logger::prelude::*;
use starcoin_node::NodeHandle;
use starcoin_rpc_client::RpcClient;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use std::sync::Arc;

#[derive(Default)]
pub struct MyWorld {
    ipc_path: Option<String>,
    storage: Option<Storage>,
    rpc_client: Option<RpcClient>,
    default_account: Option<WalletAccount>,
    txn_account: Option<WalletAccount>,
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
        .given_regex(
            r#"ipc file config "([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let path = args[1].parse().unwrap();
                info!("ipc config:{:?}", path);
                world.ipc_path = Some(path)
            },
        )
        .given("a storage", |world: &mut MyWorld, _step| {
            let cache_storage = Arc::new(CacheStorage::new());
            let db_storage = Arc::new(DBStorage::new(starcoin_config::temp_path().as_ref()));
            let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
                cache_storage,
                db_storage,
            ))
            .unwrap();
            info!("storage created!");
            world.storage = Some(storage)
        })
        .given("a rpc client", |world: &mut MyWorld, _step| {
            let path = world.ipc_path.as_ref().take().unwrap();
            let client = RpcClient::connect_ipc(path).unwrap();
            info!("rpc client created!");
            world.rpc_client = Some(client)
        })
        .given("default account", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let default_account = client.clone().wallet_default();
            world.default_account = default_account.unwrap()
        })
        .given("an account", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let account = client.clone().wallet_create("integration".parse().unwrap());
            world.txn_account = Some(account.unwrap())
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
use starcoin_wallet_api::WalletAccount;
use steps::*;

cucumber! {
    features: "./features", // Path to our feature files
    world: World, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        crate::steps, // the `steps!` macro creates a `steps` function in a module
        transaction::steps,
        node::steps
    ],
    setup: setup, // Optional; called once before everything
    before: &[
        a_before_fn // Optional; called before each scenario
    ],
    after: &[
        an_after_fn // Optional; called after each scenario
    ]
}
