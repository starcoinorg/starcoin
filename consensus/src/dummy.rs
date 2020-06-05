// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::dev::DummyHeader;
use anyhow::Result;
use config::NodeConfig;
use rand::prelude::*;
use std::sync::Arc;
use traits::ChainReader;
use traits::Consensus;
use types::block::BlockHeader;
use types::U256;

#[derive(Clone)]
pub struct DummyConsensus {}

impl Consensus for DummyConsensus {
    type ConsensusHeader = DummyHeader;

    fn calculate_next_difficulty(config: Arc<NodeConfig>, _reader: &dyn ChainReader) -> U256 {
        let mut rng = rand::thread_rng();
        // if produce block on demand, use a default wait time.
        let high: u64 = if config.miner.dev_period == 0 {
            1000
        } else {
            config.miner.dev_period * 1000
        };
        let time: u64 = rng.gen_range(1, high);
        time.into()
    }

    fn solve_consensus_header(_header_hash: &[u8], _difficulty: U256) -> Self::ConsensusHeader {
        DummyHeader {}
    }

    fn verify(
        _config: Arc<NodeConfig>,
        _reader: &dyn ChainReader,
        _header: &BlockHeader,
    ) -> Result<()> {
        Ok(())
    }
}
