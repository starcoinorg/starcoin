// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .given("sync rpc client", |world: &mut MyWorld, _step| {
            let node_config = world.node_config.as_ref().take().unwrap();
            let client = RpcClient::connect_ipc(node_config.clone().rpc.get_ipc_file()).unwrap();
            info!("node local rpc client created!");
            world.rpc_client2 = Some(Arc::new(client))
        })
        .then("basic check", |world: &mut MyWorld, _step| {
            let client = world.default_rpc_client.as_ref().take().unwrap();
            let local_client = world.rpc_client2.as_ref().take().unwrap();
            let status = client.clone().node_status();
            assert!(status.is_ok());
            let list_block = client.chain_get_blocks_by_number(None, 1).unwrap();
            let max_num = list_block[0].header.number.0;
            let local_max_block = local_client
                .chain_get_block_by_number(max_num, None)
                .unwrap();
            assert!(local_max_block.is_some());
            assert_eq!(local_max_block.unwrap(), list_block[0]);
        })
        .then("node stop", |world: &mut MyWorld, _step| {
            thread::sleep(Duration::from_secs(5));
            let handle = world.node_handle.take().unwrap();
            let result = handle.stop();
            assert!(result.is_ok());
        });
    builder.build()
}
