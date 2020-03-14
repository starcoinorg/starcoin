// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::headblock_pacemaker::HeadBlockPacemaker;
use crate::ondemand_pacemaker::OndemandPacemaker;
use crate::schedule_pacemaker::SchedulePacemaker;
use actix::prelude::*;
use anyhow::Result;
use bus::BusActor;
use chain::BlockChain;
use config::{NodeConfig, PacemakerStrategy};
use consensus::Consensus;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use futures::channel::mpsc;
use logger::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use storage::BlockChainStore;
use traits::{ChainAsyncService, TxPoolAsyncService};
use types::transaction::TxStatus;

mod headblock_pacemaker;
mod miner;
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
    C: Consensus + Sync + Send + 'static,
    E: TransactionExecutor + Sync + Send + 'static,
    P: TxPoolAsyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: BlockChainStore + Sync + Send + 'static,
{
    config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    txpool: P,
    storage: Arc<S>,
    phantom_c: PhantomData<C>,
    phantom_e: PhantomData<E>,
    chain: CS,
}

impl<C, E, P, CS, S> MinerActor<C, E, P, CS, S>
where
    C: Consensus + Sync + Send + 'static,
    E: TransactionExecutor + Sync + Send + 'static,
    P: TxPoolAsyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: BlockChainStore + Sync + Send + 'static,
{
    pub fn launch(
        config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        storage: Arc<S>,
        txpool: P,
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
                    SchedulePacemaker::new(Duration::from_millis(10 * 1000), sender).start();
                }
            };

            // let tx_factory = TxFactoryActor::launch(txpool.clone(), Arc::clone(&storage)).unwrap();
            //
            // ctx.run_interval(Duration::from_millis(1000), move |act, _ctx| {
            //     tx_factory.do_send(GenTxEvent {});
            // });

            let gen_tx_chain = chain.clone();
            ctx.run_interval(Duration::from_millis(1000), move |_act, _ctx| {
                info!("miner call gen_tx.");
                let tmp_chain = gen_tx_chain.clone();
                Arbiter::spawn(async move {
                    if let Err(e) = tmp_chain.clone().gen_tx().await {
                        warn!("err : {:?}", e);
                    }
                });
            });

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

impl<C, E, P, CS, S> Actor for MinerActor<C, E, P, CS, S>
where
    C: Consensus + Sync + Send + 'static,
    E: TransactionExecutor + Sync + Send + 'static,
    P: TxPoolAsyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: BlockChainStore + Sync + Send + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Miner actor started");
    }
}

impl<C, E, P, CS, S> Handler<GenerateBlockEvent> for MinerActor<C, E, P, CS, S>
where
    C: Consensus + Sync + Send + 'static,
    E: TransactionExecutor + Sync + Send + 'static,
    P: TxPoolAsyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: BlockChainStore + Sync + Send + 'static,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: GenerateBlockEvent, ctx: &mut Self::Context) -> Self::Result {
        let txpool = self.txpool.clone();
        let bus = self.bus.clone();
        let storage = self.storage.clone();
        let chain = self.chain.clone();
        let config = self.config.clone();
        let f = async {
            //TODO handle error.
            let txns = txpool
                .clone()
                .get_pending_txns(None)
                .await
                .unwrap_or(vec![]);
            if !(config.miner.pacemaker_strategy == PacemakerStrategy::Ondemand && txns.is_empty())
            {
                let chain_info = chain.get_chain_info().await.unwrap();
                debug!("head block : {:?}, txn len: {}", chain_info, txns.len());
                std::thread::spawn(move || {
                    let block_chain =
                        BlockChain::<E, C, S, P>::new(config.clone(), chain_info, storage, txpool)
                            .unwrap();
                    match miner::mint::<C>(config, txns, &block_chain, bus) {
                        Err(e) => {
                            error!("mint block err: {:?}", e);
                        }
                        Ok(_) => {
                            info!("mint block success.");
                        }
                    };
                });
            }
        }
        .into_actor(self);
        ctx.spawn(f);
        Ok(())
    }
}
