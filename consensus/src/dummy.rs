// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use traits::ChainReader;
use traits::Consensus;
use types::block::BlockHeader;
use types::U256;

#[derive(Clone)]
pub struct DummyConsensus {}

impl Consensus for DummyConsensus {
    fn calculate_next_difficulty(chain: &dyn ChainReader) -> Result<U256> {
        let epoch = Self::epoch(chain)?;
        Ok(epoch.block_time_target().into())
    }

    fn solve_consensus_nonce(_header_hash: &[u8], _difficulty: U256) -> u64 {
        0
    }

    fn verify(_reader: &dyn ChainReader, _header: &BlockHeader) -> Result<()> {
        Ok(())
    }
}
