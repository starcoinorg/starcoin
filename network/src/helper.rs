// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//use anyhow::Result;
use network_p2p::PeerId;
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn convert_boot_nodes(boot_nodes: Vec<String>) -> Vec<String> {
    boot_nodes
        .iter()
        .map(|x| {
            let dx = x.rfind("/").expect("Failed to parse boot nodes");

            let peer_address = &x[dx + 1..];
            let addr = &x[..dx];
            let peer_id = PeerId::from_str(peer_address).expect("Failed to parse account address");
            format!("{:}/{:}", addr, peer_id).to_string()
        })
        .clone()
        .collect()
}

pub fn get_unix_ts() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_nanos()
}