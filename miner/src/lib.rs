// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ondemand_pacemaker::OndemandPacemaker;
use crate::schedule_pacemaker::SchedulePacemaker;
use actix::prelude::*;
use anyhow::Result;
use bus::BusActor;
use chain::{BlockChain, ChainActor};
use config::NodeConfig;
use consensus::{Consensus, ConsensusHeader};
use executor::TransactionExecutor;
use futures::channel::mpsc;
use futures::{Future, TryFutureExt};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use storage::StarcoinStorage;
use traits::{ChainReader, TxPoolAsyncService};

mod headblock_pacemaker;
mod miner;
mod ondemand_pacemaker;
mod schedule_pacemaker;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateBlockEvent {}

pub struct MinerActor<C, E, P>
where
    C: Consensus + 'static,
    E: TransactionExecutor + 'static,
    P: TxPoolAsyncService + 'static,
{
    config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    txpool: P,
    storage: Arc<StarcoinStorage>,
    phantom_c: PhantomData<C>,
    phantom_e: PhantomData<E>,
}

impl<C, E, P> MinerActor<C, E, P>
where
    C: Consensus,
    E: TransactionExecutor,
    P: TxPoolAsyncService,
{
    pub fn launch(
        config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        storage: Arc<StarcoinStorage>,
        txpool: P,
    ) -> Result<Addr<Self>> {
        let actor = MinerActor::create(move |ctx| {
            let (sender, receiver) = mpsc::channel(100);
            ///TODO create pacemaker by config.
            let pacemaker = SchedulePacemaker::new(Duration::from_millis(1000), sender);
            ctx.add_message_stream(receiver);
            pacemaker.start();
            MinerActor {
                config,
                bus,
                txpool,
                storage,
                phantom_c: PhantomData,
                phantom_e: PhantomData,
            }
        });
        Ok(actor)
    }
}

impl<C, E, P> Actor for MinerActor<C, E, P>
where
    C: Consensus,
    E: TransactionExecutor,
    P: TxPoolAsyncService,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Miner actor started");
    }
}

impl<C, E, P> Handler<GenerateBlockEvent> for MinerActor<C, E, P>
where
    C: Consensus,
    E: TransactionExecutor,
    P: TxPoolAsyncService,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: GenerateBlockEvent, ctx: &mut Self::Context) -> Self::Result {
        let txpool = self.txpool.clone();
        let bus = self.bus.clone();
        let config = self.config.clone();
        let storage = self.storage.clone();

        let f = async {
            //TODO handle error.
            let txns = txpool.get_pending_txns().await.unwrap_or(vec![]);
            //TODO load latest head block.
            let block_chain = BlockChain::<E, C>::new(config, storage, None).unwrap();
            miner::mint::<C>(txns, &block_chain, bus);
            // let block_chain =
            //     BlockChain::<E, C>::new(self.config.clone(), self.storage.clone(), None)
            //         .unwrap();
        }
        .into_actor(self);
        ctx.spawn(f);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
