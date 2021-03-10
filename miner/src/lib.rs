// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MINER_METRICS;
use crate::task::MintTask;
use anyhow::Result;
use consensus::Consensus;
use futures::executor::block_on;
use logger::prelude::*;
use starcoin_config::NodeConfig;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
    ServiceRequest,
};
use std::sync::Arc;
use std::time::Duration;

mod create_block_template;
pub mod generate_block_event_pacemaker;
pub mod job_bus_client;
mod metrics;
pub mod task;

pub use create_block_template::{CreateBlockTemplateRequest, CreateBlockTemplateService};
pub use starcoin_miner_client::miner::{MinerClient, MinerClientService};
pub use types::block::BlockHeaderExtra;
pub use types::system_events::{GenerateBlockEvent, MinedBlock, MintBlockEvent, SubmitSealEvent};

#[derive(Debug)]
pub enum MinerClientSubscribeRequest {
    Add(u32),
    Remove(u32),
}

impl ServiceRequest for MinerClientSubscribeRequest {
    type Response = Result<Option<MintBlockEvent>>;
}

pub struct MinerService {
    config: Arc<NodeConfig>,
    current_task: Option<MintTask>,
    create_block_template_service: ServiceRef<CreateBlockTemplateService>,
    client_subscribers_num: u32,
}

impl ServiceHandler<Self, MinerClientSubscribeRequest> for MinerService {
    fn handle(
        &mut self,
        msg: MinerClientSubscribeRequest,
        _ctx: &mut ServiceContext<MinerService>,
    ) -> Result<Option<MintBlockEvent>> {
        match msg {
            MinerClientSubscribeRequest::Add(num) => {
                self.client_subscribers_num = num;
                Ok(self.current_task.as_ref().map(|task| MintBlockEvent {
                    strategy: task.block_template.strategy,
                    minting_blob: task.minting_blob.clone(),
                    difficulty: task.block_template.difficulty,
                    block_number: task.block_template.number,
                }))
            }
            MinerClientSubscribeRequest::Remove(num) => {
                self.client_subscribers_num = num;
                Ok(None)
            }
        }
    }
}

impl ServiceFactory<MinerService> for MinerService {
    fn create(ctx: &mut ServiceContext<MinerService>) -> Result<MinerService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let create_block_template_service =
            ctx.service_ref::<CreateBlockTemplateService>()?.clone();
        Ok(MinerService {
            config,
            current_task: None,
            create_block_template_service,
            client_subscribers_num: 0,
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
        if let Err(e) = self.finish_task(event.nonce, event.extra, event.minting_blob.clone(), ctx)
        {
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
        if block_template.body.transactions.is_empty()
            && self.config.miner.is_disable_mint_empty_block()
        {
            debug!("The flag disable_mint_empty_block is true and no txn in pool, so skip mint empty block.");
            Ok(())
        } else {
            debug!("Mint block template: {:?}", block_template);
            let difficulty = block_template.difficulty;
            let strategy = block_template.strategy;
            let number = block_template.number;
            let task = MintTask::new(block_template);
            let mining_blob = task.minting_blob.clone();
            if let Some(current_task) = self.current_task.as_ref() {
                debug!(
                    "force set mint task, current_task: {:?}, new_task: {:?}",
                    current_task, task
                );
            }
            self.current_task = Some(task);
            ctx.broadcast(MintBlockEvent::new(
                strategy,
                mining_blob,
                difficulty,
                number,
            ));
            Ok(())
        }
    }

    pub fn finish_task(
        &mut self,
        nonce: u32,
        extra: BlockHeaderExtra,
        minting_blob: Vec<u8>,
        ctx: &mut ServiceContext<MinerService>,
    ) -> Result<()> {
        match self.current_task.as_ref() {
            None => {
                debug!(
                    "MintTask is none, but got nonce: {}, extra:{:?} for minting_blob: {:?}, may be mint by other client.",
                    nonce, extra, minting_blob,
                );
                return Ok(());
            }
            Some(task) => {
                if task.minting_blob != minting_blob {
                    info!(
                        "[miner] Jobs hash mismatch expect: {}, got: {}, probably received old job result.",
                        hex::encode(task.minting_blob.as_slice()),
                        hex::encode(minting_blob.as_slice())
                    );
                    return Ok(());
                }
                if let Err(e) = task.block_template.strategy.verify_blob(
                    task.minting_blob.clone(),
                    nonce,
                    extra,
                    task.block_template.difficulty,
                ) {
                    warn!(
                        "Failed to verify blob: {}, nonce: {}, err: {}",
                        hex::encode(task.minting_blob.as_slice()),
                        nonce,
                        e
                    );
                    return Ok(());
                }
            }
        }

        if let Some(task) = self.current_task.take() {
            let block = task.finish(nonce, extra);
            info!("Mint new block: {}", block);
            ctx.broadcast(MinedBlock(Arc::new(block)));
            MINER_METRICS.block_mint_count.inc();
        }
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
        if self.client_subscribers_num == 0 && self.config.miner.disable_miner_client() {
            debug!("No miner client connected, ignore GenerateBlockEvent.");
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
