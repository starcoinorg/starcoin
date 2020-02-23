// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libp2p::multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    // The address that this node is listening on for new connections.
    pub listen_address: Multiaddr,
    pub advertised_address: Multiaddr,
    pub listen: String,
    pub seeds: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "/ip4/0.0.0.0/tcp/9840".parse::<Multiaddr>().unwrap(),
            advertised_address: "/ip4/127.0.0.1/tcp/9840".parse::<Multiaddr>().unwrap(),
            listen:"/ip4/0.0.0.0/tcp/9840".to_string(),
            seeds:vec![],
        }
    }
}
