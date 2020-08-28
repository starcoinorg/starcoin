// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::stratum::mint;
use actix::prelude::*;
use anyhow::Result;
use bus::{BusActor, Subscription};
use chain::BlockChain;
use config::NodeConfig;
use create_block_template::{
    CreateBlockTemplateActor, CreateBlockTemplateActorAddress, CreateBlockTemplateRequest,
};
use crypto::hash::PlainCryptoHash;
use futures::prelude::*;
use logger::prelude::*;
use sc_stratum::Stratum;
use starcoin_account_api::AccountInfo;
pub use starcoin_miner_client::miner::{Miner as MinerClient, MinerClientActor};
use starcoin_txpool_api::TxPoolSyncService;
use std::cmp::min;
use std::sync::Arc;
use storage::Store;
use types::startup_info::StartupInfo;
use types::system_events::ActorStop;

mod create_block_template;
pub mod headblock_pacemaker;
mod metrics;
pub mod miner;
pub mod ondemand_pacemaker;
pub mod stratum;

use crate::create_block_template::GetHeadRequest;
pub use types::system_events::GenerateBlockEvent;

pub struct MinerActor<P, S>
where
    P: TxPoolSyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    txpool: P,
    storage: Arc<S>,
    miner: miner::Miner,
    stratum: Arc<Stratum>,
    miner_account: AccountInfo,
    arbiter: Arbiter,
    create_block_template_address: CreateBlockTemplateActorAddress,
}

impl<P, S> MinerActor<P, S>
where
    P: TxPoolSyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    pub fn launch(
        config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        storage: Arc<S>,
        txpool: P,
        miner_account: AccountInfo,
        startup_info: StartupInfo,
    ) -> Result<Addr<Self>> {
        let create_block_template_address = CreateBlockTemplateActor::launch(
            *startup_info.get_master(),
            config.net(),
            bus.clone(),
            storage.clone(),
        )?;
        let actor = MinerActor::create(move |_ctx| {
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
                bus,
                txpool,
                storage,
                miner,
                stratum,
                miner_account,
                arbiter,
                create_block_template_address,
            }
        });
        Ok(actor)
    }
}

impl<P, S> Actor for MinerActor<P, S>
where
    P: TxPoolSyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<GenerateBlockEvent>();
        self.bus
            .clone()
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        info!("MinerActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("MinerActor stopped");
    }
}

impl<P, S> Handler<ActorStop> for MinerActor<P, S>
where
    P: TxPoolSyncService + Sync + Send + 'static,
    S: Store + Sync + Send + 'static,
{
    type Result = ();

    fn handle(&mut self, _msg: ActorStop, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop()
    }
}

impl<P, S> Handler<GenerateBlockEvent> for MinerActor<P, S>
where
    P: TxPoolSyncService + Sync + Send + 'static,
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
        let config = self.config.clone();
        let miner = self.miner.clone();
        let stratum = self.stratum.clone();
        let miner_account = self.miner_account.clone();

        let enable_mint_empty_block = self.config.miner.enable_mint_empty_block;
        let self_address = ctx.address();
        let create_block_template_address = self.create_block_template_address.clone();
        let f = async move {
            let head = create_block_template_address.send(GetHeadRequest{}).await??.head;
            let block_chain = BlockChain::new(config.net().consensus(), head, storage.clone(), None)?;
            let on_chain_block_gas_limit = block_chain.get_on_chain_block_gas_limit()?;
            let block_gas_limit = config.miner.block_gas_limit.map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit)).unwrap_or(on_chain_block_gas_limit);
            //TODO use a GasConstant value to replace 600.
            // block_gas_limit / min_gas_per_txn
            let max_txns = block_gas_limit / 600;

            let txns = txpool.get_pending_txns(Some(max_txns), None);

            debug!(
                "On GenerateBlockEvent, head: {:?}, block_gas_limit: {}, max_txns: {}, txn len: {}",
                head,
                block_gas_limit,
                max_txns,
                txns.len()
            );

            if txns.is_empty() && !enable_mint_empty_block {
                debug!("The flag enable_mint_empty_block is false and no txn in pool, so skip mint empty block.");
                Ok(())
            } else {
                let final_block_gas_limit = config.miner.block_gas_limit
                    .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
                    .unwrap_or(on_chain_block_gas_limit);
                let response = create_block_template_address.send(CreateBlockTemplateRequest::new(final_block_gas_limit, *miner_account.address(),
                                                              Some(miner_account.get_auth_key().prefix().to_vec()),txns))
                    .await??;

                let (block_template, excluded_txns) = response.into();

                for invalid_txn in excluded_txns.discarded_txns {
                    let _ = txpool.remove_txn(invalid_txn.crypto_hash(), true);
                }

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
