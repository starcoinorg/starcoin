// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::miner::{MineCtx, Miner};
use config::ConsensusStrategy;
use consensus::Consensus;
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
    fn submit(&self, mut payload: Vec<String>) -> Result<(), Error> {
        self.miner
            .submit(payload.pop().unwrap_or_default())
            .map_err(Error::Dispatch)?;
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
    let difficulty =
        strategy.calculate_next_difficulty(chain, &ConsensusStrategy::epoch(chain)?)?;
    //TODO refactor miner and job dispatch.
    miner.set_mint_job(MineCtx::new(block_template, difficulty));
    let job = miner.get_mint_job().expect("Mint job should exist.");
    debug!("Push job to worker {}", job);
    if let Err(e) = stratum.push_work_all(job) {
        error!("Stratum push failed:{:?}", e);
    }
    Ok(())
}
