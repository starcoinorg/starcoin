// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MINER_METRICS;
use actix::prelude::*;
use anyhow::{format_err, Result};
use bus::{Broadcast, BusActor};
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
    metrics_timer: Option<HistogramTimer>,
}

impl MineCtx {
    pub fn new(block_template: BlockTemplate, difficulty: U256) -> MineCtx {
        let header_hash = block_template.as_raw_block_header(difficulty).crypto_hash();
        let metrics_timer = Some(
            MINER_METRICS
                .block_mint_time
                .with_label_values(&["mint"])
                .start_timer(),
        );
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
    pub fn set_mint_job(&mut self, t: MineCtx) {
        let mut state = self.state.lock().unwrap();
        *state = Some(t)
    }

    pub fn get_mint_job(&mut self) -> String {
        let state = self.state.lock().unwrap();
        let x = state.as_ref().unwrap().to_owned();
        format!(r#"["{:x}","{:x}"]"#, x.header_hash, x.difficulty)
    }

    pub fn submit(&self, payload: String) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let metrics_timer = state.as_mut().unwrap().metrics_timer.take();
        let block_template = state.as_ref().unwrap().block_template.clone();
        let nonce = u64::from_str_radix(&payload, 16).map_err(|e| {
            format_err!(
                "Invalid payload submit: {}, decode failed:{}",
                payload,
                e.to_string()
            )
        })?;
        let difficulty = state.as_ref().unwrap().difficulty;
        let block = block_template.into_block(nonce, difficulty);
        info!("Miner new block with id: {:?}", block.id());
        self.bus.do_send(Broadcast {
            msg: MinedBlock(Arc::new(block)),
        });
        MINER_METRICS.block_mint_count.inc();
        if let Some(timer) = metrics_timer {
            timer.observe_duration();
        }
        Ok(())
    }
}
