// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod consensus;

use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use network::NetworkActor;

pub struct ConsensusActor {}

impl ConsensusActor {
    pub fn launch(
        _node_config: &NodeConfig,
        _network: Addr<NetworkActor>,
    ) -> Result<Addr<ConsensusActor>> {
        let actor = ConsensusActor {};
        Ok(actor.start())
    }
}

impl Actor for ConsensusActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Consensus actor started");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
