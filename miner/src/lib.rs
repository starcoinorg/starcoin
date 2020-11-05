// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MINER_METRICS;
use crate::task::MintTask;
use anyhow::{format_err, Result};
use chain::BlockChain;
use consensus::Consensus;
use crypto::HashValue;
use futures::executor::block_on;
use logger::prelude::*;
use starcoin_config::NodeConfig;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::Storage;
use std::sync::Arc;
use std::time::Duration;
use traits::ChainReader;
use types::system_events::{SyncBegin, SyncDone};

mod create_block_template;
pub mod headblock_pacemaker;
pub mod job_bus_client;
mod metrics;
pub mod ondemand_pacemaker;
pub mod task;

pub use create_block_template::{CreateBlockTemplateRequest, CreateBlockTemplateService};

pub use starcoin_miner_client::miner::{MinerClient, MinerClientService};

pub use types::system_events::{GenerateBlockEvent, MinedBlock, MintBlockEvent, SubmitSealEvent};

pub struct MinerService {
    config: Arc<NodeConfig>,
    storage: Arc<Storage>,
    current_task: Option<MintTask>,
    create_block_template_service: ServiceRef<CreateBlockTemplateService>,
    generate_block: bool,
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
            generate_block: false,
        })
    }
}

impl ActorService for MinerService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<GenerateBlockEvent>();
        ctx.subscribe::<SubmitSealEvent>();
        ctx.subscribe::<SyncBegin>();
        ctx.subscribe::<SyncDone>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<GenerateBlockEvent>();
        ctx.unsubscribe::<SubmitSealEvent>();
        ctx.unsubscribe::<SyncBegin>();
        ctx.unsubscribe::<SyncDone>();
        Ok(())
    }
}

impl EventHandler<Self, SubmitSealEvent> for MinerService {
    fn handle_event(&mut self, event: SubmitSealEvent, ctx: &mut ServiceContext<MinerService>) {
        if let Err(e) = self.finish_task(event.nonce, event.header_hash, ctx) {
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
            let mining_hash = task.mining_hash;
            if self.is_minting() {
                warn!("force set mint task, since mint task is not empty");
            }
            self.current_task = Some(task);
            ctx.broadcast(MintBlockEvent::new(
                block_chain.consensus(),
                mining_hash,
                difficulty,
            ));
            Ok(())
        }
    }

    pub fn finish_task(
        &mut self,
        nonce: u64,
        header_hash: HashValue,
        ctx: &mut ServiceContext<MinerService>,
    ) -> Result<()> {
        let task = self.current_task.take().ok_or_else(|| {
            format_err!(
                "MintTask is none, but got nonce: {} for header_hash: {:?}",
                nonce,
                header_hash
            )
        })?;
        if task.mining_hash != header_hash {
            warn!(
                "Header hash mismatch expect: {:?}, got: {:?}, probably received old job result.",
                task.mining_hash, header_hash
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
        if !self.generate_block {
            debug!("Ignore generate block event, wait sync finished.");
            return;
        }
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

impl EventHandler<Self, SyncBegin> for MinerService {
    fn handle_event(&mut self, _msg: SyncBegin, _ctx: &mut ServiceContext<MinerService>) {
        self.generate_block = false;
    }
}

impl EventHandler<Self, SyncDone> for MinerService {
    fn handle_event(&mut self, _msg: SyncDone, ctx: &mut ServiceContext<MinerService>) {
        self.generate_block = true;
        ctx.notify(GenerateBlockEvent::new(false));
    }
}
