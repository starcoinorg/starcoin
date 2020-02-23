// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::miner::Miner;
use crate::ondemand_pacemaker::OndemandPacemaker;
use crate::schedule_pacemaker::SchedulePacemaker;
use actix::prelude::*;
use anyhow::Result;
use bus::BusActor;
use chain::ChainActor;
use config::NodeConfig;
use consensus::{Consensus, ConsensusHeader};
use futures::channel::mpsc;
use futures::{Future, TryFutureExt};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use traits::{ChainReader, TxPoolAsyncService};

mod headblock_pacemaker;
mod miner;
mod ondemand_pacemaker;
mod schedule_pacemaker;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateBlockEvent {}

pub struct MinerActor<C, P>
where
    C: Consensus + 'static,
    P: TxPoolAsyncService + 'static,
{
    miner: Miner<C>,
    txpool: P,
}

impl<C, P> MinerActor<C, P>
where
    C: Consensus,
    P: TxPoolAsyncService,
{
    pub fn launch(
        _node_config: &NodeConfig,
        bus: Addr<BusActor>,
        chain_reader: Arc<dyn ChainReader>,
        txpool: P,
    ) -> Result<Addr<Self>> {
        let actor = MinerActor::create(move |ctx| {
            let (sender, receiver) = mpsc::channel(100);
            ///TODO create pacemaker by config.
            let pacemaker = SchedulePacemaker::new(Duration::from_millis(1000), sender);
            ctx.add_message_stream(receiver);
            pacemaker.start();
            MinerActor {
                miner: Miner::new(bus, chain_reader),
                txpool,
            }
        });
        Ok(actor)
    }
}

impl<C, P> Actor for MinerActor<C, P>
where
    C: Consensus,
    P: TxPoolAsyncService,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Miner actor started");
    }
}

impl<C, P> Handler<GenerateBlockEvent> for MinerActor<C, P>
where
    C: Consensus,
    P: TxPoolAsyncService,
{
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, _event: GenerateBlockEvent, ctx: &mut Self::Context) -> Self::Result {
        let f = self
            .txpool
            .clone()
            .get_pending_txns()
            .and_then(|result| async move {
                //self.miner.mint(result)?;
                Ok(())
            });
        Box::new(actix::fut::wrap_future(f))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
