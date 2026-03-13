// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MinerMetrics;
use crate::task::MintTask;
use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
    ServiceRequest,
};
use starcoin_types::block::BlockTemplate;
use std::sync::Arc;
use std::time::Duration;

mod create_block_template;
pub mod generate_block_event_pacemaker;
mod metrics;
pub mod task;

use crate::create_block_template::BlockTemplateResponse;
pub use create_block_template::{BlockBuilderService, BlockTemplateRequest};
use starcoin_crypto::HashValue;
pub use starcoin_types::block::BlockHeaderExtra;
pub use starcoin_types::system_events::{GenerateBlockEvent, MinedBlock, MintBlockEvent};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MinerError {
    #[error("Mint task is empty Error")]
    TaskEmptyError,
    #[error("Mint task is mismatch Error, current blob: {current}, got blob: {real}")]
    TaskMisMatchError { current: String, real: String },
}

#[derive(Debug)]
pub struct UpdateSubscriberNumRequest {
    pub number: Option<u32>,
}

impl ServiceRequest for UpdateSubscriberNumRequest {
    type Response = Option<MintBlockEvent>;
}

pub struct MinerService {
    config: Arc<NodeConfig>,
    current_task: Option<MintTask>,
    create_block_template_service: ServiceRef<BlockBuilderService>,
    client_subscribers_num: u32,
    inflight_template_build: bool,
    pending_template_event: Option<PendingGenerateBlockEvent>,
    metrics: Option<MinerMetrics>,
}

impl ServiceRequest for SubmitSealRequest {
    type Response = Result<HashValue>;
}

#[derive(Clone, Debug)]
pub struct SubmitSealRequest {
    pub nonce: u32,
    pub extra: BlockHeaderExtra,
    pub minting_blob: Vec<u8>,
}

impl fmt::Display for SubmitSealRequest {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "Seal{{nonce:{}, extra:{}, blob:{}}}",
            self.nonce,
            self.extra,
            hex::encode(&self.minting_blob)
        )
    }
}

impl SubmitSealRequest {
    pub fn new(minting_blob: Vec<u8>, nonce: u32, extra: BlockHeaderExtra) -> Self {
        Self {
            minting_blob,
            nonce,
            extra,
        }
    }
}

impl ServiceHandler<Self, UpdateSubscriberNumRequest> for MinerService {
    fn handle(
        &mut self,
        req: UpdateSubscriberNumRequest,
        ctx: &mut ServiceContext<MinerService>,
    ) -> Option<MintBlockEvent> {
        let previous = self.client_subscribers_num;
        if let Some(num) = req.number {
            self.client_subscribers_num = num;
            if previous == 0
                && self.client_subscribers_num > 0
                && self.current_task.is_none()
                && !self.inflight_template_build
            {
                ctx.notify(GenerateBlockEvent::default());
            }
        }
        self.current_task.as_ref().map(|task| MintBlockEvent {
            parent_hash: task.block_template.parent_hash,
            strategy: task.block_template.strategy,
            minting_blob: task.minting_blob.clone(),
            difficulty: task.block_template.difficulty,
            block_number: task.block_template.number,
            extra: None,
        })
    }
}

impl ServiceFactory<MinerService> for MinerService {
    fn create(ctx: &mut ServiceContext<MinerService>) -> Result<MinerService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let create_block_template_service = ctx.service_ref::<BlockBuilderService>()?.clone();
        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| MinerMetrics::register(registry).ok());
        Ok(MinerService {
            config,
            current_task: None,
            create_block_template_service,
            client_subscribers_num: 0,
            inflight_template_build: false,
            pending_template_event: None,
            metrics,
        })
    }
}

impl ActorService for MinerService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<GenerateBlockEvent>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<GenerateBlockEvent>();
        Ok(())
    }
}

impl ServiceHandler<Self, SubmitSealRequest> for MinerService {
    fn handle(
        &mut self,
        req: SubmitSealRequest,
        ctx: &mut ServiceContext<MinerService>,
    ) -> Result<HashValue> {
        self.finish_task(req.nonce, req.extra, req.minting_blob.clone(), ctx)
            .map_err(|e| {
                warn!(target: "miner", "process seal: {} failed: {}", req, e);
                e
            })
    }
}

// one hour
const MAX_BLOCK_TIME_GAP: u64 = 3600 * 1000;

#[derive(Clone, Copy, Debug, Default)]
struct PendingGenerateBlockEvent {
    break_current_task: bool,
    skip_empty_block_check: bool,
}

impl PendingGenerateBlockEvent {
    fn merge(&mut self, event: GenerateBlockEvent) {
        self.break_current_task |= event.break_current_task;
        self.skip_empty_block_check |= event.skip_empty_block_check;
    }

    fn into_event(self) -> GenerateBlockEvent {
        GenerateBlockEvent::new(self.break_current_task, self.skip_empty_block_check)
    }
}

impl From<GenerateBlockEvent> for PendingGenerateBlockEvent {
    fn from(event: GenerateBlockEvent) -> Self {
        Self {
            break_current_task: event.break_current_task,
            skip_empty_block_check: event.skip_empty_block_check,
        }
    }
}

#[derive(Clone, Debug)]
struct BlockTemplateBuilt {
    request: PendingGenerateBlockEvent,
    response: std::result::Result<BlockTemplateResponse, String>,
}

impl MinerService {
    fn queue_template_build(&mut self, event: GenerateBlockEvent) {
        match self.pending_template_event.as_mut() {
            Some(pending) => pending.merge(event),
            None => self.pending_template_event = Some(event.into()),
        }
    }

    fn maybe_start_template_build(&mut self, ctx: &mut ServiceContext<MinerService>) {
        if self.inflight_template_build {
            return;
        }

        let request = match self.pending_template_event.take() {
            Some(request) => request,
            None => return,
        };
        self.inflight_template_build = true;

        let create_block_template_service = self.create_block_template_service.clone();
        let self_ref = ctx.self_ref();
        ctx.spawn(async move {
            let response = match create_block_template_service
                .send(BlockTemplateRequest)
                .await
            {
                Ok(Ok(response)) => Ok(response),
                Ok(Err(err)) => Err(err.to_string()),
                Err(err) => Err(err.to_string()),
            };

            if let Err(err) = self_ref.notify(BlockTemplateBuilt { request, response }) {
                warn!(
                    target: "miner",
                    "failed to notify block template result: {:?}",
                    err
                );
            }
        });
    }

    fn handle_block_template_built(
        &mut self,
        ctx: &mut ServiceContext<MinerService>,
        request: PendingGenerateBlockEvent,
        response: BlockTemplateResponse,
    ) -> Result<()> {
        let parent = response.parent;
        let block_template = response.template;
        let block_time_gap = block_template.timestamp - parent.timestamp();

        if !request.skip_empty_block_check
            && (block_template.body.transactions.is_empty()
            && self.config.miner.is_disable_mint_empty_block()
            //if block time gap > 3600, force create a empty block for fix https://github.com/starcoinorg/starcoin/issues/3036
            && block_time_gap < MAX_BLOCK_TIME_GAP)
        {
            debug!("The flag disable_mint_empty_block is true and no txn in pool, so skip mint empty block.");
            Ok(())
        } else {
            self.dispatch_mint_block_event(ctx, block_template)
        }
    }

    fn dispatch_mint_block_event(
        &mut self,
        ctx: &mut ServiceContext<MinerService>,
        block_template: BlockTemplate,
    ) -> Result<()> {
        debug!("Mint block template: {:?}", block_template);
        let difficulty = block_template.difficulty;
        let strategy = block_template.strategy;
        let number = block_template.number;
        let parent_hash = block_template.parent_hash;
        let task = MintTask::new(block_template, self.metrics.clone());
        let mining_blob = task.minting_blob.clone();
        if let Some(current_task) = self.current_task.as_ref() {
            debug!(
                "force set mint task, current_task: {:?}, new_task: {:?}",
                current_task, task
            );
        }
        self.current_task = Some(task);
        ctx.broadcast(MintBlockEvent::new(
            parent_hash,
            strategy,
            mining_blob,
            difficulty,
            number,
            None,
        ));
        Ok(())
    }

    pub fn finish_task(
        &mut self,
        nonce: u32,
        extra: BlockHeaderExtra,
        minting_blob: Vec<u8>,
        ctx: &mut ServiceContext<MinerService>,
    ) -> Result<HashValue> {
        match self.current_task.as_ref() {
            Some(task) => {
                if task.minting_blob != minting_blob {
                    return Err(MinerError::TaskMisMatchError {
                        current: hex::encode(&task.minting_blob),
                        real: hex::encode(minting_blob),
                    }
                    .into());
                };
                task.block_template.strategy.verify_blob(
                    task.minting_blob.clone(),
                    nonce,
                    extra,
                    task.block_template.difficulty,
                )?
            }
            None => {
                return Err(MinerError::TaskEmptyError.into());
            }
        }

        if let Some(task) = self.current_task.take() {
            let block = task.finish(nonce, extra);
            let block_hash = block.id();
            info!(target: "miner", "Mint new block: {}", block);
            ctx.broadcast(MinedBlock(Arc::new(block)));
            if let Some(metrics) = self.metrics.as_ref() {
                metrics.block_mint_count.inc();
            }
            Ok(block_hash)
        } else {
            Err(MinerError::TaskEmptyError.into())
        }
    }

    pub fn is_minting(&self) -> bool {
        self.current_task.is_some()
    }
}

impl EventHandler<Self, GenerateBlockEvent> for MinerService {
    fn handle_event(&mut self, event: GenerateBlockEvent, ctx: &mut ServiceContext<MinerService>) {
        debug!("Handle GenerateBlockEvent:{:?}", event);
        if !event.break_current_task && self.is_minting() {
            debug!("Miner has mint job so just ignore this event.");
            return;
        }
        if self.config.miner.disable_miner_client() && self.client_subscribers_num == 0 {
            debug!("No miner client connected, ignore GenerateBlockEvent.");
            // Once Miner client connect, we should dispatch task.
            ctx.run_later(Duration::from_secs(2), |ctx| {
                ctx.notify(GenerateBlockEvent::default());
            });
            return;
        }
        self.queue_template_build(event);
        self.maybe_start_template_build(ctx);
    }
}

impl EventHandler<Self, BlockTemplateBuilt> for MinerService {
    fn handle_event(&mut self, event: BlockTemplateBuilt, ctx: &mut ServiceContext<MinerService>) {
        self.inflight_template_build = false;

        match event.response {
            Ok(response) => {
                if let Err(err) = self.handle_block_template_built(ctx, event.request, response) {
                    warn!(
                        target: "miner",
                        "Failed to process generated block template: {}, delay to trigger a new event.",
                        err
                    );
                    let retry_event = event.request.into_event();
                    ctx.run_later(Duration::from_secs(2), move |ctx| {
                        ctx.notify(retry_event.clone());
                    });
                }
            }
            Err(err) => {
                warn!(
                    target: "miner",
                    "Failed to build block template: {}, delay to trigger a new event.",
                    err
                );
                let retry_event = event.request.into_event();
                ctx.run_later(Duration::from_secs(2), move |ctx| {
                    ctx.notify(retry_event.clone());
                });
            }
        }

        self.maybe_start_template_build(ctx);
    }
}
