// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use logger::prelude::*;
use rand::prelude::*;
use std::convert::TryFrom;
use std::thread;
use std::time::{Duration, SystemTime};
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
        info!("epoch: {:?}", epoch);
        let current_header = chain.current_header();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();
        //in dev mode, if disable_empty_block = true,
        //may escape a long time between block,
        //so, just set the difficulty to 1 for sleep less time for this case.
        let target =
            (now as i64) - (current_header.timestamp as i64) - (epoch.block_time_target() as i64);
        let target = if target >= 0 { 1 } else { target.abs() * 1000 };

        Ok(target.into())
    }

    fn solve_consensus_header(_header_hash: &[u8], difficulty: U256) -> Self::ConsensusHeader {
        let mut rng = rand::thread_rng();
        let time: u64 = rng.gen_range(1, difficulty.as_u64() * 2);
        info!(
            "DevConsensus rand sleep time in millis second : {}, difficulty : {}",
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
