// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use config::NodeConfig;
use network::Network;

pub struct Consensus {
    network: Addr<Network>,
}

impl Consensus {
    pub fn new(_node_config: &NodeConfig, network: Addr<Network>) -> Self {
        Consensus { network }
    }
}

impl Actor for Consensus {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {}
}
