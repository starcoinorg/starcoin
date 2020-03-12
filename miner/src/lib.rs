// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::headblock_pacemaker::HeadBlockPacemaker;
use crate::ondemand_pacemaker::OndemandPacemaker;
use crate::schedule_pacemaker::SchedulePacemaker;
use crate::tx_factory::{GenTxEvent, TxFactoryActor};
use actix::prelude::*;
use anyhow::Result;
use bus::BusActor;
use chain::{BlockChain, BlockChainStore, ChainActor, ChainActorRef};
use config::{NodeConfig, PacemakerStrategy};
use consensus::{Consensus, ConsensusHeader};
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use futures::channel::mpsc;
use futures::{Future, TryFutureExt,};
use futures::prelude::*;
use logger::prelude::*;
use starcoin_accumulator::AccumulatorNodeStore;
use state_tree::StateNodeStore;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use storage::{BlockStorageOp, StarcoinStorage};
use traits::{ChainAsyncService, ChainReader, TxPoolAsyncService};
use types::transaction::TxStatus;
use crate::miner::Miner;

mod headblock_pacemaker;
mod miner;
mod stratum;
mod ondemand_pacemaker;
mod schedule_pacemaker;
#[cfg(test)]
mod tests;
mod tx_factory;

pub(crate) type TransactionStatusEvent = Arc<Vec<(HashValue, TxStatus)>>;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateBlockEvent {}

pub struct MinerActor<C, E, P, CS, S>
where
    C: Consensus + 'static,
    E: TransactionExecutor + 'static,
    P: TxPoolAsyncService + 'static,
    CS: ChainAsyncService + 'static,
    S: BlockChainStore + 'static,
{
    config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    txpool: P,
    storage: Arc<S>,
    phantom_c: PhantomData<C>,
    phantom_e: PhantomData<E>,
    chain: CS,
    miner: Miner,
}

impl<C, E, P, CS, S> MinerActor<C, E, P, CS, S>
where
    C: Consensus,
    E: TransactionExecutor,
    P: TxPoolAsyncService,
    CS: ChainAsyncService,
    S: BlockChainStore + 'static,
{
    pub fn launch(
        config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        storage: Arc<S>,
        mut txpool: P,
        chain: CS,
        mut transaction_receiver: Option<mpsc::UnboundedReceiver<TransactionStatusEvent>>,
    ) -> Result<Addr<Self>> {
        let actor = MinerActor::create(move |ctx| {
            let (sender, receiver) = mpsc::channel(100);
            ctx.add_message_stream(receiver);
            match &config.miner.pacemaker_strategy {
                PacemakerStrategy::HeadBlock => {
                    HeadBlockPacemaker::new(bus.clone(), sender).start();
                }
                PacemakerStrategy::Ondemand => {
                    OndemandPacemaker::new(
                        bus.clone(),
                        sender.clone(),
                        transaction_receiver.take().unwrap(),
                    )
                    .start();
                }
                PacemakerStrategy::Schedule => {
                    SchedulePacemaker::new(Duration::from_millis(1000), sender).start();
                }
            };

            // let tx_factory = TxFactoryActor::launch(txpool.clone(), Arc::clone(&storage)).unwrap();
            //
            // ctx.run_interval(Duration::from_millis(1000), move |act, _ctx| {
            //     tx_factory.do_send(GenTxEvent {});
            // });

            let gen_tx_chain = chain.clone();
            ctx.run_interval(Duration::from_millis(1000), move |act, _ctx| {
                info!("miner call gen_tx.");
                let tmp_chain = gen_tx_chain.clone();
                Arbiter::spawn(async move {
                    tmp_chain.clone().gen_tx().await;
                });
            });
            let miner = Miner::new();
            MinerActor {
                config,
                bus,
                txpool,
                storage,
                phantom_c: PhantomData,
                phantom_e: PhantomData,
                chain,
                miner,
            }
        });
        Ok(actor)
    }
}

impl<C, E, P, CS, S> Actor for MinerActor<C, E, P, CS, S>
where
    C: Consensus,
    E: TransactionExecutor,
    P: TxPoolAsyncService,
    CS: ChainAsyncService,
    S: BlockChainStore + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Miner actor started");
    }
}

impl<C, E, P, CS, S> Handler<GenerateBlockEvent> for MinerActor<C, E, P, CS, S>
where
    C: Consensus,
    E: TransactionExecutor,
    P: TxPoolAsyncService,
    CS: ChainAsyncService,
    S: BlockChainStore + 'static,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: GenerateBlockEvent, ctx: &mut Self::Context) -> Self::Result {
        let txpool_1 = self.txpool.clone();
        let txpool_2 = self.txpool.clone();
        let bus = self.bus.clone();
        let config = self.config.clone();
        let storage = self.storage.clone();
        let chain = self.chain.clone();
        let mut miner = self.miner.clone();

        let f = async move{
            //TODO handle error.
            let txns = txpool_1.get_pending_txns(None).await.unwrap_or(vec![]);
            if !(config.miner.pacemaker_strategy == PacemakerStrategy::Ondemand && txns.is_empty())
            {
                //TODO load latest head block.
                let head_branch = chain.get_head_branch().await;
                info!("head block : {:?}, txn len: {}", head_branch, txns.len());
                let block_chain =
                    BlockChain::<E, C, S, P>::new(config, storage, head_branch, txpool_2).unwrap();
                let block_template = block_chain.create_block_template(txns).unwrap();
                miner.set_mint_job(block_template);
                // stratum.update_all_worker()
            }
        }
        .into_actor(self);
        ctx.spawn(f);
        Ok(())
    }
}
