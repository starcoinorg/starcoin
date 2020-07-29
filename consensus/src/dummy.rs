// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::time::{MockTimeService, TimeService};
use anyhow::Result;
use logger::prelude::*;
use rand::Rng;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::on_chain_config::EpochInfo;

#[derive(Default)]
pub struct DummyConsensus {
    time_service: MockTimeService,
}

impl DummyConsensus {
    pub fn new() -> Self {
        let s = Self {
            time_service: MockTimeService::new(),
        };
        // 0 is genesis time, auto increment to 1.
        s.time_service.increment();
        s
    }
}

impl Consensus for DummyConsensus {
    fn calculate_next_difficulty(&self, chain: &dyn ChainReader,epoch: &EpochInfo) -> Result<U256> {
        let epoch = Self::epoch(chain)?;
        Ok(epoch.block_time_target().into())
    }

    fn solve_consensus_nonce(&self, _header_hash: &[u8], difficulty: U256) -> u64 {
        let mut rng = rand::thread_rng();
        let time: u64 = rng.gen_range(1, difficulty.as_u64() * 2);
        debug!(
            "DummyConsensus rand sleep time in millis second : {}, difficulty : {}",
            time,
            difficulty.as_u64()
        );
        self.time_service.sleep(time);
        time
    }

    fn verify(&self, _reader: &dyn ChainReader, _epoch: &EpochInfo, _header: &BlockHeader) -> Result<()> {
        Ok(())
    }

    fn time(&self) -> &dyn TimeService {
        &self.time_service
    }
}
