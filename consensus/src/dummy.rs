// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::dev::DummyHeader;
use anyhow::Result;
use traits::ChainReader;
use traits::Consensus;
use types::block::BlockHeader;
use types::U256;

#[derive(Clone)]
pub struct DummyConsensus {}

impl Consensus for DummyConsensus {
    type ConsensusHeader = DummyHeader;

    fn calculate_next_difficulty(chain: &dyn ChainReader) -> Result<U256> {
        let epoch = Self::epoch(chain)?;
        Ok(epoch.block_difficulty_window().into())
    }

    fn solve_consensus_header(_header_hash: &[u8], _difficulty: U256) -> Self::ConsensusHeader {
        DummyHeader {}
    }

    fn verify(_reader: &dyn ChainReader, _header: &BlockHeader) -> Result<()> {
        Ok(())
    }
}
