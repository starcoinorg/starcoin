// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MINER_METRICS;
use crate::MintBlockEvent;
use actix::prelude::*;
use anyhow::{format_err, Result};
use bus::{Bus, BusActor};
use config::NodeConfig;
use crypto::hash::PlainCryptoHash;
use crypto::HashValue;
use logger::prelude::*;
use starcoin_metrics::HistogramTimer;
use std::sync::Arc;
use std::sync::Mutex;
use types::{block::BlockTemplate, system_events::MinedBlock, U256};

#[derive(Clone)]
pub struct Miner {
    state: Arc<Mutex<Option<MineCtx>>>,
    bus: Addr<BusActor>,
    config: Arc<NodeConfig>,
}

pub struct MineCtx {
    header_hash: HashValue,
    block_template: BlockTemplate,
    difficulty: U256,
    metrics_timer: HistogramTimer,
}

impl MineCtx {
    pub fn new(block_template: BlockTemplate, difficulty: U256) -> MineCtx {
        let header_hash = block_template.as_raw_block_header(difficulty).crypto_hash();
        let metrics_timer = MINER_METRICS
            .block_mint_time
            .with_label_values(&["mint"])
            .start_timer();
        MineCtx {
            header_hash,
            block_template,
            difficulty,
            metrics_timer,
        }
    }
}

impl Miner {
    pub fn new(bus: Addr<BusActor>, config: Arc<NodeConfig>) -> Miner {
        Self {
            state: Arc::new(Mutex::new(None)),
            bus,
            config,
        }
    }

    pub async fn set_mint(&self, block_template: BlockTemplate, difficulty: U256) -> Result<()> {
        let ctx = MineCtx::new(block_template, difficulty);
        let header_hash = ctx.header_hash;
        if self.is_minting() {
            warn!("force set mint job, since mint ctx is not empty");
        }
        *self.state.lock().unwrap() = Some(ctx);
        let bus = self.bus.clone();
        bus.broadcast(MintBlockEvent::new(header_hash, difficulty))
            .await
    }

    pub fn is_minting(&self) -> bool {
        self.state.lock().unwrap().is_some()
    }

    pub async fn submit(&self, nonce: u64, header_hash: HashValue) -> Result<()> {
        let ctx = self
            .state
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| format_err!("Empty mine ctx"))?;
        if ctx.header_hash != header_hash {
            self.reset_ctx(Some(ctx));
            return Err(format_err!("Header hash mismatch"));
        }
        let block = ctx.block_template.into_block(nonce, ctx.difficulty);
        self.reset_ctx(None);
        info!("Mint new block with id: {:?}", block.id());
        self.bus
            .clone()
            .broadcast(MinedBlock(Arc::new(block)))
            .await?;
        MINER_METRICS.block_mint_count.inc();
        ctx.metrics_timer.observe_duration();
        Ok(())
    }

    fn reset_ctx(&self, ctx: Option<MineCtx>) {
        *(self.state.lock().unwrap()) = ctx;
    }
}
