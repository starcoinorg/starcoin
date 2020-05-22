// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::headblock_pacemaker::HeadBlockPacemaker;
use crate::ondemand_pacemaker::OndemandPacemaker;
use crate::schedule_pacemaker::SchedulePacemaker;
use crate::stratum::mint;
use actix::prelude::*;
use anyhow::Result;
use bus::BusActor;
use chain::BlockChain;
use config::{ConsensusStrategy, NodeConfig, PacemakerStrategy};
use crypto::hash::HashValue;
use futures::channel::mpsc;
use futures::prelude::*;
use logger::prelude::*;
pub use miner_client::miner::{Miner as MinerClient, MinerClientActor};
use sc_stratum::Stratum;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_wallet_api::WalletAccount;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use storage::Store;
use traits::ChainAsyncService;
use traits::Consensus;
use types::transaction::TxStatus;

mod headblock_pacemaker;
mod miner;
mod miner_client;
mod ondemand_pacemaker;
mod schedule_pacemaker;
mod stratum;

pub(crate) type TransactionStatusEvent = Arc<Vec<(HashValue, TxStatus)>>;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateBlockEvent {}

pub struct MinerActor<C, P, CS, S>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolSyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    config: Arc<NodeConfig>,
    txpool: P,
    storage: Arc<S>,
    phantom_c: PhantomData<C>,
    chain: CS,
    miner: miner::Miner<C>,
    stratum: Arc<Stratum>,
    miner_account: WalletAccount,
    arbiter: Arbiter,
}

impl<C, P, CS, S> MinerActor<C, P, CS, S>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolSyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    pub fn launch(
        config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        storage: Arc<S>,
        txpool: P,
        chain: CS,
        miner_account: WalletAccount,
    ) -> Result<Addr<Self>> {
        let actor = MinerActor::create(move |ctx| {
            let (sender, receiver) = mpsc::channel(100);
            ctx.add_message_stream(receiver);
            match &config.miner.pacemaker_strategy {
                PacemakerStrategy::HeadBlock => {
                    let pacemaker = HeadBlockPacemaker::new(bus.clone(), sender);
                    pacemaker.start();
                }
                PacemakerStrategy::Ondemand => {
                    let transaction_receiver = txpool.subscribe_txns();

                    OndemandPacemaker::new(bus.clone(), sender, transaction_receiver).start();
                }
                PacemakerStrategy::Schedule => match config.miner.consensus_strategy {
                    ConsensusStrategy::Dummy(dev_period) => {
                        SchedulePacemaker::new(Duration::from_secs(dev_period), sender).start();
                    }
                    ConsensusStrategy::Argon(_) => {
                        panic!("Incompatible consensus strategy");
                    }
                },
            };

            let miner = miner::Miner::new(bus.clone(), config.clone());

            let stratum = sc_stratum::Stratum::start(
                &config.miner.stratum_server,
                Arc::new(stratum::StratumManager::new(miner.clone())),
                None,
            )
            .unwrap();
            let arbiter = Arbiter::new();
            MinerActor {
                config,
                txpool,
                storage,
                phantom_c: PhantomData,
                chain,
                miner,
                stratum,
                miner_account,
                arbiter,
            }
        });
        Ok(actor)
    }
}

impl<C, P, CS, S> Actor for MinerActor<C, P, CS, S>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolSyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Miner actor started");
    }
}

impl<C, P, CS, S> Handler<GenerateBlockEvent> for MinerActor<C, P, CS, S>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolSyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: GenerateBlockEvent, _ctx: &mut Self::Context) -> Self::Result {
        let txpool = self.txpool.clone();
        let storage = self.storage.clone();
        let chain = self.chain.clone();
        let config = self.config.clone();
        let miner = self.miner.clone();
        let stratum = self.stratum.clone();
        let miner_account = self.miner_account.clone();
        // block_gas_limit / min_gas_per_txn
        let max_txns = self.config.miner.block_gas_limit / 600;
        let f = async move {
            let txns = txpool.get_pending_txns(Some(max_txns));
            let startup_info = chain.master_startup_info().await?;
            debug!(
                "On GenerateBlockEvent, master: {:?}, txn len: {}",
                startup_info.master,
                txns.len()
            );
            let master = *startup_info.get_master();
            let block_chain = BlockChain::<C, S>::new(config.clone(), master, storage)?;
            mint::<C>(stratum, miner, config, miner_account, txns, &block_chain)?;
            Ok(())
        }
        .map(|result: Result<()>| {
            if let Err(err) = result {
                error!("Failed to process generate block event:{:?}", err)
            }
        });
        self.arbiter.send(Box::pin(f));
        Ok(())
    }
}
