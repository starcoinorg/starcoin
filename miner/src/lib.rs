// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MinerMetrics;
use crate::task::MintTask;
use anyhow::Result;
use futures::executor::block_on;
use starcoin_config::NodeConfig;
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

pub use create_block_template::{BlockBuilderService, BlockTemplateRequest};
use starcoin_crypto::HashValue;
pub use starcoin_types::block::BlockHeaderExtra;
pub use starcoin_types::system_events::{GenerateBlockEvent, MinedBlock, MintBlockEvent};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

const DEFAULT_TASK_POOL_SIZE: usize = 16;
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
    task_pool: Vec<MintTask>,
    create_block_template_service: ServiceRef<BlockBuilderService>,
    client_subscribers_num: u32,
    metrics: Option<MinerMetrics>,
    task_flag: Arc<AtomicBool>,
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
        _ctx: &mut ServiceContext<Self>,
    ) -> Option<MintBlockEvent> {
        if let Some(num) = req.number {
            self.client_subscribers_num = num;
        }
        self.task_pool.last().map(|task| MintBlockEvent {
            parent_hash: task.block_template.parent_hash,
            strategy: task.block_template.strategy,
            minting_blob: task.minting_blob.clone(),
            difficulty: task.block_template.difficulty,
            block_number: task.block_template.number,
            extra: None,
        })
    }
}

impl ServiceFactory<Self> for MinerService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let create_block_template_service = ctx.service_ref::<BlockBuilderService>()?.clone();
        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| MinerMetrics::register(registry).ok());
        Ok(Self {
            config,
            task_pool: Vec::new(),
            create_block_template_service,
            client_subscribers_num: 0,
            metrics,
            task_flag: Arc::new(AtomicBool::new(false)),
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
        info!("stoped miner_serive ");
        Ok(())
    }
}

impl ServiceHandler<Self, SubmitSealRequest> for MinerService {
    fn handle(
        &mut self,
        req: SubmitSealRequest,
        ctx: &mut ServiceContext<Self>,
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

#[derive(Debug)]
pub struct SyncBlockTemplateRequest {
    pub respond_to: futures::channel::oneshot::Sender<Result<Option<BlockTemplate>>>,
}
impl ServiceRequest for SyncBlockTemplateRequest {
    type Response = ();
}
impl ServiceHandler<Self, SyncBlockTemplateRequest> for MinerService {
    fn handle(&mut self, msg: SyncBlockTemplateRequest, ctx: &mut ServiceContext<Self>) {
        let config = self.config.clone();
        let create_block_template_service = self.create_block_template_service.clone();
        let tx = msg.respond_to;

        ctx.spawn(async move {
            let result = async {
                let res = create_block_template_service
                    .send(BlockTemplateRequest)
                    .await
                    .map_err(|e| anyhow::anyhow!("send BlockTemplateRequest failed: {}", e))??;

                let parent = res.parent;
                let block_template = res.template;
                let block_time_gap = block_template.timestamp - parent.timestamp();

                if block_template.body.transactions.is_empty()
                    && config.miner.is_disable_mint_empty_block()
                    && block_time_gap < MAX_BLOCK_TIME_GAP
                {
                    Ok(None)
                } else {
                    Ok(Some(block_template))
                }
            }
            .await;

            let _ = tx.send(result); // send back
        });
    }
}

#[derive(Debug)]
pub struct DispatchMintBlockTemplate {
    pub block_template: BlockTemplate,
}

impl ServiceRequest for DispatchMintBlockTemplate {
    type Response = ();
}

impl ServiceHandler<Self, DispatchMintBlockTemplate> for MinerService {
    fn handle(&mut self, msg: DispatchMintBlockTemplate, ctx: &mut ServiceContext<Self>) {
        let _ = self.dispatch_mint_block_event(ctx, msg.block_template.clone());
    }
}

#[derive(Debug)]
pub struct DelayGenerateBlockEvent {
    pub delay_secs: u64,
}

impl ServiceRequest for DelayGenerateBlockEvent {
    type Response = ();
}

impl ServiceHandler<Self, DelayGenerateBlockEvent> for MinerService {
    fn handle(&mut self, msg: DelayGenerateBlockEvent, ctx: &mut ServiceContext<Self>) {
        ctx.run_later(Duration::from_secs(msg.delay_secs), |ctx| {
            ctx.notify(GenerateBlockEvent::default());
        });
    }
}

impl MinerService {
    pub fn dispatch_task(
        &mut self,
        ctx: &mut ServiceContext<Self>,
        event: GenerateBlockEvent,
    ) -> anyhow::Result<()> {
        if self.task_flag.load(Ordering::Relaxed) {
            debug!("Mint task already running, skip dispatch");
            return Ok(());
        }
        self.task_flag.store(true, Ordering::Relaxed);

        let create_block_template_service = self.create_block_template_service.clone();
        let config = self.config.clone();
        let addr = ctx.service_ref::<Self>()?.clone();
        let flag = self.task_flag.clone();
        ctx.spawn(async move {
            let result = tokio::time::timeout(
                Duration::from_millis(2000),
                create_block_template_service.send(BlockTemplateRequest),
            )
            .await;
            match result {
                Ok(inner) => match inner {
                    Ok(Ok(response)) => {
                        let parent = response.parent;
                        let block_template = response.template;
                        let block_time_gap = block_template.timestamp - parent.timestamp();

                        let should_skip = !event.skip_empty_block_check
                            && block_template.body.transactions.is_empty()
                            && config.miner.is_disable_mint_empty_block()
                            && block_time_gap < 3600 * 1000;

                        if !should_skip {
                            let _ = addr
                                .send(DispatchMintBlockTemplate { block_template })
                                .await;
                        } else {
                            info!("Skipping minting empty block");
                        }
                    }
                    Ok(Err(e)) => {
                        warn!("BlockTemplateRequest failed: {}", e);
                        let _ = addr.send(DelayGenerateBlockEvent { delay_secs: 2 }).await;
                    }
                    Err(e) => {
                        warn!("ServiceRef send failed: {}", e);
                        let _ = addr.send(DelayGenerateBlockEvent { delay_secs: 2 }).await;
                    }
                },
                Err(elapsed) => {
                    warn!(
                        "Timeout waiting BlockTemplateRequest: {}. Retrying.",
                        elapsed
                    );
                    let _ = addr.send(DelayGenerateBlockEvent { delay_secs: 2 }).await;
                }
            }
            flag.store(false, Ordering::Relaxed);
        });

        Ok(())
    }

    pub fn dispatch_sleep_task(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        //create block template should block_on for avoid mint same block template.
        let response = block_on(async {
            self.create_block_template_service
                .send(BlockTemplateRequest)
                .await?
        })?;
        self.dispatch_mint_block_event(ctx, response.template)
    }

    fn dispatch_mint_block_event(
        &mut self,
        ctx: &mut ServiceContext<Self>,
        block_template: BlockTemplate,
    ) -> Result<()> {
        debug!("Mint block template: {:?}", block_template);
        let difficulty = block_template.difficulty;
        let strategy = block_template.strategy;
        let number = block_template.number;
        let parent_hash = block_template.parent_hash;
        let task = MintTask::new(block_template, self.metrics.clone());
        let mining_blob = task.minting_blob.clone();
        self.task_pool.retain(|t| t.minting_blob != mining_blob);
        self.manage_task_pool();
        self.task_pool.push(task);
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
        ctx: &mut ServiceContext<Self>,
    ) -> Result<HashValue> {
        if self.task_pool.is_empty() {
            return Err(MinerError::TaskEmptyError.into());
        }
        if let Some(index) = self
            .task_pool
            .iter()
            .position(|t| t.minting_blob == minting_blob)
        {
            let task = self.task_pool.remove(index);

            let block = task.finish(nonce, extra);
            let block_hash: HashValue = block.id();
            info!(target: "miner", "Minted new block: {}", block);
            ctx.broadcast(MinedBlock(Arc::new(block)));
            if let Some(metrics) = self.metrics.as_ref() {
                metrics.block_mint_count.inc();
            }

            Ok(block_hash)
        } else {
            // TODO:: Refactor TaskMisMatchError,remove current @sanlee
            Err(MinerError::TaskMisMatchError {
                current: "None".to_string(),
                real: hex::encode(&minting_blob),
            }
            .into())
        }
    }

    pub fn is_minting(&self) -> bool {
        !self.task_pool.is_empty()
    }
    fn manage_task_pool(&mut self) {
        if self.task_pool.len() > DEFAULT_TASK_POOL_SIZE {
            self.task_pool.remove(0);
        }
    }
    pub fn task_pool_len(&self) -> usize {
        self.task_pool.len()
    }
    pub fn get_task_pool(&self) -> &Vec<MintTask> {
        &self.task_pool
    }
}

impl EventHandler<Self, GenerateBlockEvent> for MinerService {
    fn handle_event(&mut self, event: GenerateBlockEvent, ctx: &mut ServiceContext<Self>) {
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
        if let Err(err) = self.dispatch_task(ctx, event) {
            warn!(
                "Failed to process generate block event:{}, delay to trigger a new event.",
                err
            );
            ctx.run_later(Duration::from_secs(2), move |ctx| {
                ctx.notify(GenerateBlockEvent::default());
            });
        }
    }
}
