// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    headblock_pacemaker::HeadBlockPacemaker, ondemand_pacemaker::OndemandPacemaker, stratum::mint,
};
use actix::prelude::*;
use anyhow::Result;
use bus::BusActor;
use chain::BlockChain;
use config::NodeConfig;
use crypto::hash::HashValue;
use futures::{channel::mpsc, prelude::*};
use logger::prelude::*;
use sc_stratum::Stratum;
use starcoin_account_api::AccountInfo;
pub use starcoin_miner_client::miner::{Miner as MinerClient, MinerClientActor};
use starcoin_txpool_api::TxPoolSyncService;
use std::cmp::min;
use std::sync::Arc;
use storage::Store;
use traits::ChainAsyncService;
use types::transaction::TxStatus;

mod headblock_pacemaker;
mod metrics;
pub mod miner;
mod ondemand_pacemaker;
pub mod stratum;

pub(crate) type TransactionStatusEvent = Arc<Vec<(HashValue, TxStatus)>>;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateBlockEvent {
    /// Force break current minting, and Generate new block.
    force: bool,
}

impl GenerateBlockEvent {
    pub fn new(force: bool) -> Self {
        Self { force }
    }
}

pub struct MinerActor<P, CS, S>
where
    P: TxPoolSyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    config: Arc<NodeConfig>,
    txpool: P,
    storage: Arc<S>,
    chain: CS,
    miner: miner::Miner,
    stratum: Arc<Stratum>,
    miner_account: AccountInfo,
    arbiter: Arbiter,
}

impl<P, CS, S> MinerActor<P, CS, S>
where
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
        miner_account: AccountInfo,
    ) -> Result<Addr<Self>> {
        let actor = MinerActor::create(move |ctx| {
            let (sender, receiver) = mpsc::channel(100);
            ctx.add_message_stream(receiver);
            let pacemaker = HeadBlockPacemaker::new(bus.clone(), sender.clone());
            pacemaker.start();
            let transaction_receiver = txpool.subscribe_txns();
            OndemandPacemaker::new(bus.clone(), sender, transaction_receiver).start();

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

impl<P, CS, S> Actor for MinerActor<P, CS, S>
where
    P: TxPoolSyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("MinerActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("MinerActor stopped");
    }
}

impl<P, CS, S> Handler<GenerateBlockEvent> for MinerActor<P, CS, S>
where
    P: TxPoolSyncService + Sync + Send + 'static,
    CS: ChainAsyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    type Result = Result<()>;

    fn handle(&mut self, event: GenerateBlockEvent, ctx: &mut Self::Context) -> Self::Result {
        debug!("Handle GenerateBlockEvent:{:?}", event);
        if !event.force && self.miner.has_mint_job() {
            debug!("Miner has mint job so just ignore this event.");
            return Ok(());
        }
        let txpool = self.txpool.clone();
        let storage = self.storage.clone();
        let chain = self.chain.clone();
        let config = self.config.clone();
        let miner = self.miner.clone();
        let stratum = self.stratum.clone();
        let miner_account = self.miner_account.clone();

        let enable_mint_empty_block = self.config.miner.enable_mint_empty_block;
        let self_address = ctx.address();
        let f = async move {
            let startup_info = chain.master_startup_info().await?;
            let master = *startup_info.get_master();
            let block_chain = BlockChain::new(config.net(), master, storage.clone(), None)?;
            let on_chain_block_gas_limit = block_chain.get_on_chain_block_gas_limit()?;
            let block_gas_limit = config.miner.block_gas_limit.map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit)).unwrap_or(on_chain_block_gas_limit);
            //TODO use a GasConstant value to replace 600.
            // block_gas_limit / min_gas_per_txn
            let max_txns = block_gas_limit / 600;

            let txns = txpool.get_pending_txns(Some(max_txns), None);

            debug!(
                "On GenerateBlockEvent, master: {:?}, block_gas_limit: {}, max_txns: {}, txn len: {}",
                startup_info.master,
                block_gas_limit,
                max_txns,
                txns.len()
            );

            if txns.is_empty() && !enable_mint_empty_block {
                debug!("The flag enable_mint_empty_block is false and no txn in pool, so skip mint empty block.");
                Ok(())
            } else {
                let block_template = chain
                    .create_block_template(
                        *miner_account.address(),
                        Some(miner_account.get_auth_key().prefix().to_vec()),
                        None,
                        txns,
                    )
                    .await?;

                mint(stratum, miner, config.net().consensus(), &block_chain, block_template)?;
                Ok(())
            }
        }.map(move |result: Result<()>| {
            if let Err(err) = result {
                error!("Failed to process generate block event:{:?}, try to fire a new event.", err);
                if let Err(send_error) = self_address.try_send(GenerateBlockEvent::new(false)) {
                    error!("Failed send GenerateBlockEvent: {:?}", send_error);
                };
            }
        });
        self.arbiter.send(Box::pin(f));
        Ok(())
    }
}
