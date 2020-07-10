// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use logger::prelude::*;
use rand::prelude::*;
use std::convert::TryFrom;
use std::thread;
use std::time::Duration;
use traits::ChainReader;
use traits::{Consensus, ConsensusHeader};
use types::block::BlockHeader;
use types::U256;

//TODO add some field to DummyHeader.
#[derive(Clone, Debug)]
pub struct DummyHeader {}

impl ConsensusHeader for DummyHeader {}

impl TryFrom<Vec<u8>> for DummyHeader {
    type Error = Error;

    fn try_from(_value: Vec<u8>) -> Result<Self> {
        Ok(DummyHeader {})
    }
}

impl Into<Vec<u8>> for DummyHeader {
    fn into(self) -> Vec<u8> {
        vec![]
    }
}

#[derive(Clone)]
pub struct DevConsensus {}

impl Consensus for DevConsensus {
    type ConsensusHeader = DummyHeader;

    fn calculate_next_difficulty(chain: &dyn ChainReader) -> Result<U256> {
        let epoch = Self::epoch(chain)?;
        debug!("epoch: {:?}", epoch);
        Ok(epoch.time_target().into())
    }

    fn solve_consensus_header(_header_hash: &[u8], difficulty: U256) -> Self::ConsensusHeader {
        let mut rng = rand::thread_rng();
        let time: u64 = rng.gen_range(1, difficulty.as_u64());
        debug!(
            "DevConsensus rand sleep time : {}, difficulty : {}",
            time,
            difficulty.as_u64()
        );
        thread::sleep(Duration::from_millis(time));
        DummyHeader {}
    }

    fn verify(_reader: &dyn ChainReader, _header: &BlockHeader) -> Result<()> {
        Ok(())
    }
}
