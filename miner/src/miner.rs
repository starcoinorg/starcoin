// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MINER_METRICS;
use crate::MintBlockEvent;
use anyhow::{format_err, Result};
use crypto::hash::PlainCryptoHash;
use crypto::HashValue;
use logger::prelude::*;
use parking_lot::Mutex;
use starcoin_config::NodeConfig;
use starcoin_metrics::HistogramTimer;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use std::sync::Arc;
use types::{block::BlockTemplate, system_events::MinedBlock, U256};

#[derive(Clone)]
pub struct Miner {
    state: Arc<Mutex<Option<MineCtx>>>,
    bus: ServiceRef<BusService>,
    config: Arc<NodeConfig>,
}

pub struct MineCtx {
    mining_hash: HashValue,
    block_template: BlockTemplate,
    difficulty: U256,
    metrics_timer: HistogramTimer,
}

impl MineCtx {
    pub fn new(block_template: BlockTemplate, difficulty: U256) -> MineCtx {
        let mining_hash = block_template.as_raw_block_header(difficulty).crypto_hash();
        let metrics_timer = MINER_METRICS
            .block_mint_time
            .with_label_values(&["mint"])
            .start_timer();
        MineCtx {
            mining_hash,
            block_template,
            difficulty,
            metrics_timer,
        }
    }
}

impl Miner {
    pub fn new(bus: ServiceRef<BusService>, config: Arc<NodeConfig>) -> Miner {
        Self {
            state: Arc::new(Mutex::new(None)),
            bus,
            config,
        }
    }

    pub async fn set_mint(&self, block_template: BlockTemplate, difficulty: U256) -> Result<()> {
        let ctx = MineCtx::new(block_template, difficulty);
        let mining_hash = ctx.mining_hash;
        if self.is_minting() {
            warn!("force set mint job, since mint ctx is not empty");
        }
        *self.state.lock() = Some(ctx);
        self.bus
            .broadcast(MintBlockEvent::new(mining_hash, difficulty))?;
        Ok(())
    }

    pub fn is_minting(&self) -> bool {
        self.state.lock().is_some()
    }

    pub async fn submit(&self, nonce: u64, header_hash: HashValue) -> Result<()> {
        let ctx = self
            .state
            .lock()
            .take()
            .ok_or_else(|| format_err!("Mint job is empty"))?;
        debug!("miner receive submit with hash:{}", header_hash);

        if ctx.mining_hash != header_hash {
            self.reset_ctx(Some(ctx));
            return Err(format_err!("Header hash mismatch"));
        }

        let block = ctx.block_template.into_block(nonce, ctx.difficulty);
        info!("Mint new block: {}", block);
        self.bus.broadcast(MinedBlock(Arc::new(block)))?;
        MINER_METRICS.block_mint_count.inc();
        ctx.metrics_timer.observe_duration();
        Ok(())
    }

    fn reset_ctx(&self, ctx: Option<MineCtx>) {
        *(self.state.lock()) = ctx;
    }
}
