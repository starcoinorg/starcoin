// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MINER_METRICS;
use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use crypto::HashValue;
use logger::prelude::*;
use starcoin_metrics::HistogramTimer;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;
use traits::Consensus;
use types::{block::BlockTemplate, system_events::MinedBlock, U256};

#[derive(Clone)]
pub struct Miner<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    state: Arc<Mutex<Option<MineCtx>>>,
    bus: Addr<BusActor>,
    config: Arc<NodeConfig>,
    phantom: PhantomData<C>,
}

pub struct MineCtx {
    header_hash: HashValue,
    block_template: BlockTemplate,
    difficulty: U256,
    metrics_timer: Option<HistogramTimer>,
}

impl MineCtx {
    pub fn new(block_template: BlockTemplate, difficulty: U256) -> MineCtx {
        let header_hash = block_template.parent_hash;
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

impl<C> Miner<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(bus: Addr<BusActor>, config: Arc<NodeConfig>) -> Miner<C> {
        Self {
            state: Arc::new(Mutex::new(None)),
            bus,
            config,
            phantom: PhantomData,
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

        let payload = hex::decode(payload).unwrap();
        let consensus_header = match C::ConsensusHeader::try_from(payload) {
            Ok(h) => h,
            Err(_) => return Err(anyhow::anyhow!("Invalid payload submit")),
        };
        let difficulty = state.as_ref().unwrap().difficulty;
        let block = block_template.into_block(consensus_header, difficulty);
        // TODO: need to verify header to make sure the block is correctly mined.
        // if it's not ok, return early.
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
