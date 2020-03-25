// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::miner::Miner;
use consensus::ConsensusHeader;
use sc_stratum::*;
pub struct StratumManager<H>
where
    H: ConsensusHeader + Sync + Send + 'static,
{
    miner: Miner<H>,
}

impl<H> StratumManager<H>
where
    H: ConsensusHeader + Sync + Send + 'static,
{
    pub fn new(miner: Miner<H>) -> Self {
        Self { miner }
    }
}

impl<H> JobDispatcher for StratumManager<H>
where
    H: ConsensusHeader + Sync + Send + 'static,
{
    fn submit(&self, payload: Vec<String>) -> Result<(), Error> {
        self.miner.submit(payload[0].clone().into_bytes());
        Ok(())
    }
}
