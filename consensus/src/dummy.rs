// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use anyhow::Result;
use logger::prelude::*;
use rand::Rng;
use starcoin_crypto::HashValue;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::MOKE_TIME_SERVICE;
use starcoin_vm_types::on_chain_config::EpochInfo;

#[derive(Default)]
pub struct DummyConsensus {}

impl DummyConsensus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Consensus for DummyConsensus {
    fn calculate_next_difficulty(
        &self,
        chain: &dyn ChainReader,
        epoch: &EpochInfo,
    ) -> Result<U256> {
        info!("epoch: {:?}", epoch);
        let current_header = chain.current_header();
        let now = MOKE_TIME_SERVICE.now_millis();
        //in dev mode, if disable_empty_block = true,
        //may escape a long time between block,
        //so, just set the difficulty to 1 for sleep less time for this case.
        let target =
            (now as i64) - (current_header.timestamp as i64) - (epoch.block_time_target() as i64);
        let target = if target >= 0 { 1 } else { target.abs() };

        Ok(target.into())
    }

    fn solve_consensus_nonce(&self, _mining_hash: HashValue, difficulty: U256) -> u64 {
        let mut rng = rand::thread_rng();
        let time: u64 = rng.gen_range(1, difficulty.as_u64() * 2);
        info!(
            "DummyConsensus rand sleep time in millis second : {}, difficulty : {}",
            time,
            difficulty.as_u64()
        );
        MOKE_TIME_SERVICE.sleep(time);
        time
    }

    fn verify(
        &self,
        _reader: &dyn ChainReader,
        _epoch: &EpochInfo,
        _header: &BlockHeader,
    ) -> Result<()> {
        Ok(())
    }

    fn calculate_pow_hash(&self, _mining_hash: HashValue, _nonce: u64) -> Result<HashValue> {
        unreachable!()
    }
}
