// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use anyhow::Result;
use rand::Rng;
use starcoin_chain_api::ChainReader;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::block::{BlockHeader, BlockHeaderExtra};
use starcoin_types::U256;
use starcoin_vm_types::time::TimeService;

#[derive(Default)]
pub struct DummyConsensus {}

impl DummyConsensus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Consensus for DummyConsensus {
    fn calculate_next_difficulty(&self, chain: &dyn ChainReader) -> Result<U256> {
        let epoch = chain.epoch();
        info!("epoch: {:?}", epoch);
        let target = epoch.block_time_target();
        Ok(target.into())
    }

    fn solve_consensus_nonce(
        &self,
        _mining_hash: &[u8],
        difficulty: U256,
        time_service: &dyn TimeService,
    ) -> u32 {
        let mut rng = rand::thread_rng();
        let low = difficulty.as_u32() / 2;
        let high = difficulty.as_u32().saturating_add(low);
        let time: u32 = rng.gen_range(low..high);
        info!(
            "DummyConsensus rand sleep time in millis second : {}, difficulty : {}",
            time,
            difficulty.as_u32()
        );
        time_service.sleep(time as u64);
        time
    }

    fn verify(&self, _reader: &dyn ChainReader, _header: &BlockHeader) -> Result<()> {
        Ok(())
    }

    fn calculate_pow_hash(
        &self,
        _mining_hash: &[u8],
        _nonce: u32,
        _extra: &BlockHeaderExtra,
    ) -> Result<HashValue> {
        Ok(HashValue::zero())
    }
}
