// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate log;

use crate::headblock_pacemaker::HeadBlockPacemaker;
use crate::ondemand_pacemaker::OndemandPacemaker;
use crate::schedule_pacemaker::SchedulePacemaker;
use actix::prelude::*;
use anyhow::Result;
use bus::BusActor;
use chain::{BlockChain, ChainActor, ChainActorRef};
use config::{NodeConfig, PacemakerStrategy};
use consensus::{Consensus, ConsensusHeader};
use executor::TransactionExecutor;
use futures::channel::mpsc;
use futures::{Future, TryFutureExt};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use storage::StarcoinStorage;
use traits::{ChainAsyncService, ChainReader, TxPoolAsyncService};

mod headblock_pacemaker;
mod miner;
mod ondemand_pacemaker;
mod schedule_pacemaker;
#[cfg(test)]
mod tests;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateBlockEvent {}

pub struct MinerActor<C, E, P, CS>
    where
        C: Consensus + 'static,
        E: TransactionExecutor + 'static,
        P: TxPoolAsyncService + 'static,
        CS: ChainAsyncService + 'static,
{
    config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    txpool: P,
    storage: Arc<StarcoinStorage>,
    phantom_c: PhantomData<C>,
    phantom_e: PhantomData<E>,
    chain: CS,
}

impl<C, E, P, CS> MinerActor<C, E, P, CS>
    where
        C: Consensus,
        E: TransactionExecutor,
        P: TxPoolAsyncService,
        CS: ChainAsyncService,
{
    pub fn launch(
        config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        storage: Arc<StarcoinStorage>,
        mut txpool: P,
        chain: CS,
    ) -> Result<Addr<Self>> {
        // let tmp = txpool.clone();
        // let fut = async move {
        //     tmp.subscribe_txns().await.unwrap()
        // };

        // let tx = System::builder().build().block_on(fut);

        let actor = MinerActor::create(move |ctx| {
            let (sender, receiver) = mpsc::channel(100);
            ctx.add_message_stream(receiver);
            match &config.miner.pacemaker_strategy {
                PacemakerStrategy::HeadBlock => {
                    HeadBlockPacemaker::new(bus.clone(), sender).start();
                }
                PacemakerStrategy::Ondemand => {
                    let bus = bus.clone();
                    let sender = sender.clone();
                    let txpool = txpool.clone();
                    OndemandPacemaker::new(bus, sender, txpool);
                }
                PacemakerStrategy::Schedule => {
                    SchedulePacemaker::new(Duration::from_millis(1000), sender).start();
                }
            };
            MinerActor {
                config,
                bus,
                txpool,
                storage,
                phantom_c: PhantomData,
                phantom_e: PhantomData,
                chain,
            }
        });
        Ok(actor)
    }
}

impl<C, E, P, CS> Actor for MinerActor<C, E, P, CS>
    where
        C: Consensus,
        E: TransactionExecutor,
        P: TxPoolAsyncService,
        CS: ChainAsyncService,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Miner actor started");
    }
}

impl<C, E, P, CS> Handler<GenerateBlockEvent> for MinerActor<C, E, P, CS>
    where
        C: Consensus,
        E: TransactionExecutor,
        P: TxPoolAsyncService,
        CS: ChainAsyncService,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: GenerateBlockEvent, ctx: &mut Self::Context) -> Self::Result {
        let txpool = self.txpool.clone();
        let bus = self.bus.clone();
        let config = self.config.clone();
        let storage = self.storage.clone();
        let chain = self.chain.clone();

        let f = async {
            //TODO handle error.
            let txns = txpool.get_pending_txns(None).await.unwrap_or(vec![]);
            //TODO load latest head block.
            let head_branch = chain.get_head_branch().await;
            println!("head block : {:?}", head_branch);
            let block_chain = BlockChain::<E, C>::new(config, storage, head_branch).unwrap();
            miner::mint::<C>(txns, &block_chain, bus);
        }
            .into_actor(self);
        ctx.spawn(f);
        Ok(())
    }
}
