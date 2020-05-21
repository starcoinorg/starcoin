// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
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
        });
    builder.build()
}
