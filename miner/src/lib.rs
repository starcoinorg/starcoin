// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::miner::Miner;
use crate::ondemand_pacemaker::OndemandPacemaker;
use actix::prelude::*;
use anyhow::Result;
use bus::BusActor;
use chain::ChainActor;
use config::NodeConfig;
use consensus::{ChainReader, Consensus, ConsensusHeader};
use futures::channel::mpsc;
use network::NetworkActor;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

mod miner;
mod ondemand_pacemaker;
mod schedule_pacemaker;

#[derive(Default, Debug, Message)]
#[rtype(result = "()")]
pub struct GenerateBlockEvent {}

pub struct MinerActor<C>
where
    C: Consensus,
{
    miner: Miner<C>,
}

impl<C> MinerActor<C>
where
    C: Consensus + 'static,
{
    pub fn launch(
        _node_config: &NodeConfig,
        _network: Addr<NetworkActor>,
        bus: Addr<BusActor>,
        chain_reader: Arc<dyn ChainReader>,
    ) -> Result<Addr<Self>> {
        let actor = MinerActor::create(move |ctx| {
            let (sender, receiver) = mpsc::channel(100);
            ///TODO create pacemaker by config.
            let pacemaker = OndemandPacemaker::new(bus, sender);
            ctx.add_stream(receiver);
            pacemaker.start();
            MinerActor {
                miner: Miner::new(chain_reader),
            }
        });
        Ok(actor)
    }
}

impl<C> Actor for MinerActor<C>
where
    C: Consensus + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Miner actor started");
    }
}

impl<C> StreamHandler<GenerateBlockEvent> for MinerActor<C>
where
    C: Consensus + 'static,
{
    fn handle(&mut self, _event: GenerateBlockEvent, _ctx: &mut Self::Context) {
        self.miner.mint();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
