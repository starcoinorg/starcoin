// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::miner::{MineCtx, Miner};
use config::ConsensusStrategy;
use logger::prelude::*;
use sc_stratum::*;
use std::sync::Arc;
use traits::ChainReader;
use types::block::BlockTemplate;

pub struct StratumManager {
    miner: Miner,
}

impl StratumManager {
    pub fn new(miner: Miner) -> Self {
        Self { miner }
    }
}

impl JobDispatcher for StratumManager {
    fn submit(&self, payload: Vec<String>) -> Result<(), Error> {
        //todo:: error handle
        let _ = self.miner.submit(payload[0].clone());
        Ok(())
    }
}

pub fn mint(
    stratum: Arc<Stratum>,
    mut miner: Miner,
    strategy: ConsensusStrategy,
    chain: &dyn ChainReader,
    block_template: BlockTemplate,
) -> anyhow::Result<()> {
    let difficulty = consensus::calculate_next_difficulty(strategy, chain)?;
    miner.set_mint_job(MineCtx::new(block_template, difficulty));
    let job = miner.get_mint_job();
    debug!("Push job to worker {}", job);
    if let Err(e) = stratum.push_work_all(job) {
        error!("Stratum push failed:{:?}", e);
    }
    Ok(())
}
