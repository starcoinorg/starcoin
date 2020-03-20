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
use consensus::{difficult, Consensus};

use crate::miner::MineCtx;
use chain::to_block_chain_collection;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use futures::channel::mpsc;
use logger::prelude::*;
use sc_stratum::{self, PushWorkHandler};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use storage::BlockChainStore;
use traits::ChainReader;
use traits::{ChainAsyncService, TxPoolAsyncService};
use types::transaction::TxStatus;

mod headblock_pacemaker;
#[allow(dead_code)]
mod miner;
#[allow(dead_code)]
mod miner_client;
mod ondemand_pacemaker;
mod schedule_pacemaker;
mod stratum;
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
    miner: miner::Miner,
    stratum: Arc<sc_stratum::Stratum>,
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
            let miner = miner::Miner::new(bus.clone());
            let stratum = sc_stratum::Stratum::start(
                &config.miner.stratum_server,
                Arc::new(stratum::StratumManager::new(miner.clone())),
                None,
            )
            .unwrap();

            MinerActor {
                config,
                bus,
                txpool,
                storage,
                phantom_c: PhantomData,
                phantom_e: PhantomData,
                chain,
                miner,
                stratum,
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
        let mut miner = self.miner.clone();
        let stratum = self.stratum.clone();
        let f = async {
            //TODO handle error.
            let txns = txpool
                .clone()
                .get_pending_txns(None)
                .await
                .unwrap_or(vec![]);

            let startup_info = chain.master_startup_info().await.unwrap();
            debug!("head block : {:?}, txn len: {}", startup_info, txns.len());
            std::thread::spawn(move || {
                let head = startup_info.head.clone();
                let collection = to_block_chain_collection(
                    config.clone(),
                    startup_info,
                    storage.clone(),
                    txpool.clone(),
                )
                .unwrap();
                let block_chain = BlockChain::<E, C, S, P>::new(
                    config.clone(),
                    head,
                    storage,
                    txpool,
                    collection,
                )
                .unwrap();
                let difficulty = difficult::get_next_work_required(&block_chain);
                let block_template = block_chain
                    .create_block_template(None, difficulty, txns.clone())
                    .unwrap();
                miner.set_mint_job(MineCtx::new(block_template));
                let job = miner.get_mint_job();
                info!("Push job to worker{:?}", job);
                stratum.push_work_all(job).unwrap();

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
        .into_actor(self);
        ctx.spawn(f);
        Ok(())
    }
}
