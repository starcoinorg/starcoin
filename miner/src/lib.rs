// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use chain::BlockChain;
use consensus::Consensus;
use logger::prelude::*;
use starcoin_config::NodeConfig;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::Storage;
use std::sync::Arc;
use std::time::Duration;

mod create_block_template;
pub mod generate_block_event_pacemaker;
pub mod job_bus_client;
mod metrics;
pub mod task;

use crate::metrics::MINER_METRICS;
use crate::task::MintTask;
pub use create_block_template::{CreateBlockTemplateRequest, CreateBlockTemplateService};
use futures::executor::block_on;
pub use starcoin_miner_client::miner::{MinerClient, MinerClientService};
use traits::ChainReader;
pub use types::system_events::{GenerateBlockEvent, MinedBlock, MintBlockEvent, SubmitSealEvent};

pub struct MinerService {
    config: Arc<NodeConfig>,
    storage: Arc<Storage>,
    current_task: Option<MintTask>,
    create_block_template_service: ServiceRef<CreateBlockTemplateService>,
}

impl ServiceFactory<MinerService> for MinerService {
    fn create(ctx: &mut ServiceContext<MinerService>) -> Result<MinerService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let create_block_template_service =
            ctx.service_ref::<CreateBlockTemplateService>()?.clone();
        Ok(MinerService {
            config,
            storage,
            current_task: None,
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
        if let Err(e) = self.finish_task(event.nonce, event.minting_blob.clone(), ctx) {
            error!("Process SubmitSealEvent {:?} fail: {:?}", event, e);
        }
    }
}

impl MinerService {
    pub fn dispatch_task(&mut self, ctx: &mut ServiceContext<MinerService>) -> Result<()> {
        //create block template should block_on for avoid mint same block template.
        let block_template = block_on(async {
            self.create_block_template_service
                .send(CreateBlockTemplateRequest)
                .await?
        })?;
        if block_template.body.transactions.is_empty() && !self.config.miner.enable_mint_empty_block
        {
            debug!("The flag enable_mint_empty_block is false and no txn in pool, so skip mint empty block.");
            Ok(())
        } else {
            debug!("Mint block template: {:?}", block_template);
            let block_chain = BlockChain::new(
                self.config.net().time_service(),
                block_template.parent_hash,
                self.storage.clone(),
            )?;
            let epoch = block_chain.epoch_info()?;
            let difficulty = epoch
                .epoch()
                .strategy()
                .calculate_next_difficulty(&block_chain, &epoch)?;
            let task = MintTask::new(block_template, difficulty);
            let mining_blob = task.minting_blob.clone();
            if let Some(current_task) = self.current_task.as_ref() {
                debug!(
                    "force set mint task, current_task: {:?}, new_task: {:?}",
                    current_task, task
                );
            }
            self.current_task = Some(task);
            ctx.broadcast(MintBlockEvent::new(
                block_chain.consensus(),
                mining_blob,
                difficulty,
            ));
            Ok(())
        }
    }

    pub fn finish_task(
        &mut self,
        nonce: u32,
        minting_blob: Vec<u8>,
        ctx: &mut ServiceContext<MinerService>,
    ) -> Result<()> {
        let task = self.current_task.take().ok_or_else(|| {
            format_err!(
                "MintTask is none, but got nonce: {} for minting_blob: {:?}",
                nonce,
                minting_blob,
            )
        })?;

        if task.minting_blob != minting_blob {
            warn!(
                "Header hash mismatch expect: {:?}, got: {:?}, probably received old job result.",
                task.minting_blob, minting_blob
            );
            self.current_task = Some(task);
            return Ok(());
        }
        let block = task.finish(nonce);
        info!("Mint new block: {}", block);
        ctx.broadcast(MinedBlock(Arc::new(block)));
        MINER_METRICS.block_mint_count.inc();
        Ok(())
    }

    pub fn is_minting(&self) -> bool {
        self.current_task.is_some()
    }
}

impl EventHandler<Self, GenerateBlockEvent> for MinerService {
    fn handle_event(&mut self, event: GenerateBlockEvent, ctx: &mut ServiceContext<MinerService>) {
        debug!("Handle GenerateBlockEvent:{:?}", event);
        if !event.force && self.is_minting() {
            debug!("Miner has mint job so just ignore this event.");
            return;
        }
        if let Err(err) = self.dispatch_task(ctx) {
            error!(
                "Failed to process generate block event:{:?}, delay to trigger a new event.",
                err
            );
            ctx.run_later(Duration::from_secs(2), |ctx| {
                ctx.notify(GenerateBlockEvent::new(false));
            });
        }
    }
}
