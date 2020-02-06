// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use network::NetworkActor;

pub struct ConsensusActor {
    network: Addr<NetworkActor>,
}

impl ConsensusActor {
    pub fn launch(
        _node_config: &NodeConfig,
        network: Addr<NetworkActor>,
    ) -> Result<Addr<ConsensusActor>> {
        let actor = ConsensusActor { network };
        Ok(actor.start())
    }
}

impl Actor for ConsensusActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {}
}
