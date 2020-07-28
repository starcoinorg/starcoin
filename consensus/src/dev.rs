// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use anyhow::Result;
use logger::prelude::*;
use rand::prelude::*;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct DevConsensus {}

impl Consensus for DevConsensus {
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

    fn solve_consensus_nonce(_header_hash: &[u8], difficulty: U256) -> u64 {
        let mut rng = rand::thread_rng();
        let time: u64 = rng.gen_range(1, difficulty.as_u64() * 2);
        info!(
            "DevConsensus rand sleep time in millis second : {}, difficulty : {}",
            time,
            difficulty.as_u64()
        );
        thread::sleep(Duration::from_millis(time));
        time
    }

    fn verify(_reader: &dyn ChainReader, _header: &BlockHeader) -> Result<()> {
        Ok(())
    }
}
