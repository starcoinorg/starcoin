// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use chain::BlockChain;
use consensus::Consensus;
use futures::FutureExt;
use futures_timer::Delay;
use logger::prelude::*;
use starcoin_config::ConsensusStrategy;
use starcoin_config::NodeConfig;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::Storage;
use std::sync::Arc;
use std::time::Duration;

mod create_block_template;
pub mod headblock_pacemaker;
pub mod job_bus_client;
mod metrics;
pub mod miner;
pub mod ondemand_pacemaker;

pub use create_block_template::{CreateBlockTemplateRequest, CreateBlockTemplateService};
pub use starcoin_miner_client::miner::{MinerClient, MinerClientService};
pub use types::system_events::{GenerateBlockEvent, MintBlockEvent, SubmitSealEvent};

pub struct MinerService {
    config: Arc<NodeConfig>,
    storage: Arc<Storage>,
    miner: miner::Miner,
    create_block_template_service: ServiceRef<CreateBlockTemplateService>,
}

impl ServiceFactory<MinerService> for MinerService {
    fn create(ctx: &mut ServiceContext<MinerService>) -> Result<MinerService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let bus = ctx.bus_ref().clone();
        let miner = miner::Miner::new(bus, config.clone());
        let create_block_template_service =
            ctx.service_ref::<CreateBlockTemplateService>()?.clone();
        Ok(MinerService {
            config,
            storage,
            miner,
            create_block_template_service,
        })
    }
}

impl ActorService for MinerService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<GenerateBlockEvent>();
        ctx.subscribe::<SubmitSealEvent>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<GenerateBlockEvent>();
        ctx.unsubscribe::<SubmitSealEvent>();
        Ok(())
    }
}

impl EventHandler<Self, SubmitSealEvent> for MinerService {
    fn handle_event(&mut self, event: SubmitSealEvent, ctx: &mut ServiceContext<MinerService>) {
        let miner = self.miner.clone();
        let fut = async move {
            if let Err(e) = miner.submit(event.nonce, event.header_hash).await {
                warn!("Failed to submit seal: {}", e);
            }
        };
        ctx.wait(fut);
    }
}

impl EventHandler<Self, GenerateBlockEvent> for MinerService {
    fn handle_event(&mut self, event: GenerateBlockEvent, ctx: &mut ServiceContext<MinerService>) {
        info!("Handle GenerateBlockEvent:{:?}", event);
        if !event.force && self.miner.is_minting() {
            info!("Miner has mint job so just ignore this event.");
            return;
        }
        let storage = self.storage.clone();
        let config = self.config.clone();
        let miner = self.miner.clone();

        let enable_mint_empty_block = self.config.miner.enable_mint_empty_block;
        let create_block_template_service = self.create_block_template_service.clone();
        let self_ref = ctx.self_ref();
        let f = async move {
            let block_template = create_block_template_service.send(
                CreateBlockTemplateRequest)
                .await??;

            if block_template.body.transactions.is_empty() && !enable_mint_empty_block {
                debug!("The flag enable_mint_empty_block is false and no txn in pool, so skip mint empty block.");
                Ok(())
            } else {
                debug!("Mint block template: {:?}", block_template);
                let net = config.net();
                let strategy = net.consensus();
                let block_chain = BlockChain::new(strategy, net.time_service(),block_template.parent_hash, storage)?;
                let epoch = ConsensusStrategy::epoch(&block_chain)?;
                let difficulty =
                    strategy.calculate_next_difficulty(&block_chain, &epoch)?;
                miner.set_mint(block_template, difficulty).await?;
                Ok(())
            }
        }.then(|result: Result<()>| async move {
            if let Err(err) = result {
                error!("Failed to process generate block event:{:?}, delay to trigger a new event.", err);
                Delay::new(Duration::from_millis(1000)).await;
                if let Err(err) = self_ref.notify(GenerateBlockEvent::new(false)) {
                    error!("Send self generate block event notify failed:{:?}.", err);
                }
            }
        });
        ctx.spawn(f);
    }
}
