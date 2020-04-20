// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::headblock_pacemaker::HeadBlockPacemaker;
use crate::ondemand_pacemaker::OndemandPacemaker;
use crate::schedule_pacemaker::SchedulePacemaker;
use crate::stratum::mint;
use actix::prelude::*;
use anyhow::Result;
use bus::BusActor;
use chain::to_block_chain_collection;
use chain::BlockChain;
use config::{NodeConfig, PacemakerStrategy};
use crypto::hash::HashValue;
use futures::channel::mpsc;
use futures::prelude::*;
use logger::prelude::*;
pub use miner_client::miner::{Miner as MinerClient, MinerClientActor};
use sc_stratum::Stratum;
use starcoin_txpool_api::TxPoolAsyncService;
use starcoin_wallet_api::WalletAccount;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use storage::Store;
use traits::ChainAsyncService;
use traits::{Consensus, ConsensusHeader};
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

pub struct MinerActor<C, P, CS, S, H>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolAsyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
    H: ConsensusHeader + Sync + Send + 'static,
{
    config: Arc<NodeConfig>,
    txpool: P,
    storage: Arc<S>,
    phantom_c: PhantomData<C>,
    chain: CS,
    miner: miner::Miner<H>,
    stratum: Arc<Stratum>,
    miner_account: WalletAccount,
}

impl<C, P, CS, S, H> MinerActor<C, P, CS, S, H>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolAsyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
    H: ConsensusHeader + Sync + Send + 'static,
{
    pub fn launch(
        config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        storage: Arc<S>,
        txpool: P,
        chain: CS,
        mut transaction_receiver: Option<mpsc::UnboundedReceiver<TransactionStatusEvent>>,
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
                    OndemandPacemaker::new(
                        bus.clone(),
                        sender.clone(),
                        transaction_receiver.take().unwrap(),
                    )
                    .start();
                }
                PacemakerStrategy::Schedule => {
                    SchedulePacemaker::new(Duration::from_secs(config.miner.dev_period), sender)
                        .start();
                }
            };

            let miner = miner::Miner::new(bus.clone(), config.clone());

            let stratum = sc_stratum::Stratum::start(
                &config.miner.stratum_server,
                Arc::new(stratum::StratumManager::new(miner.clone())),
                None,
            )
            .unwrap();
            MinerActor {
                config,
                txpool,
                storage,
                phantom_c: PhantomData,
                chain,
                miner,
                stratum,
                miner_account,
            }
        });
        Ok(actor)
    }
}

impl<C, P, CS, S, H> Actor for MinerActor<C, P, CS, S, H>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolAsyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
    H: ConsensusHeader + Sync + Send + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Miner actor started");
    }
}

impl<C, P, CS, S, H> Handler<GenerateBlockEvent> for MinerActor<C, P, CS, S, H>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolAsyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
    H: ConsensusHeader + Sync + Send + 'static,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: GenerateBlockEvent, ctx: &mut Self::Context) -> Self::Result {
        let txpool = self.txpool.clone();
        let storage = self.storage.clone();
        let chain = self.chain.clone();
        let config = self.config.clone();
        let miner = self.miner.clone();
        let stratum = self.stratum.clone();
        let miner_account = self.miner_account.clone();
        let f = async {
            //TODO handle error.
            let txns = txpool
                .clone()
                .get_pending_txns(None)
                .await
                .unwrap_or(vec![]);

            let startup_info = chain.master_startup_info().await?;
            debug!("head block : {:?}, txn len: {}", startup_info, txns.len());
            let master = startup_info.master.clone();
            let collection = to_block_chain_collection(
                config.clone(),
                startup_info,
                storage.clone(),
                txpool.clone(),
            )?;
            let block_chain =
                BlockChain::<C, S, P>::new(config.clone(), master, storage, txpool, collection)?;
            mint::<H, C>(stratum, miner, config, miner_account, txns, &block_chain)?;
            Ok(())
        }
        .map(|result: Result<()>| {
            if let Err(err) = result {
                error!("Failed to process generate block event:{:?}", err)
            }
        })
        .into_actor(self);
        ctx.spawn(f);
        Ok(())
    }
}
