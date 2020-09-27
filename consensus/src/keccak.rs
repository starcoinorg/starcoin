// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::time::{RealTimeService, TimeService};
use crate::{
    difficult_to_target, difficulty, generate_nonce, set_header_nonce, target_to_difficulty,
};
use anyhow::Result;
use sha3::{Digest, Keccak256};
use starcoin_crypto::HashValue;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::on_chain_config::EpochInfo;

#[derive(Default)]
pub struct KeccakConsensus {
    time_service: RealTimeService,
}

impl KeccakConsensus {
    pub fn new() -> Self {
        Self {
            time_service: RealTimeService::new(),
        }
    }
}

impl Consensus for KeccakConsensus {
    fn calculate_next_difficulty(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
    ) -> Result<U256> {
        let target = difficulty::get_next_work_required(reader, epoch)?;
        Ok(target_to_difficulty(target))
    }

    fn solve_consensus_nonce(&self, mining_hash: HashValue, difficulty: U256) -> u64 {
        let mut nonce = generate_nonce();
        loop {
            let pow_hash: U256 = self
                .calculate_pow_hash(mining_hash, nonce)
                .expect("calculate hash should work")
                .into();
            let target = difficult_to_target(difficulty);
            if pow_hash > target {
                nonce += 1;
                continue;
            }
            break;
        }
        nonce
    }

    fn verify(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
        header: &BlockHeader,
    ) -> Result<()> {
        let difficulty = self.calculate_next_difficulty(reader, epoch)?;
        self.verify_header_difficulty(difficulty, header)
    }

    /// Double keccak256 for pow hash
    fn calculate_pow_hash(&self, mining_hash: HashValue, nonce: u64) -> Result<HashValue> {
        let mix_hash = set_header_nonce(&mining_hash.to_vec(), nonce);
        let pow_hash = Keccak256::digest(Keccak256::digest(&mix_hash).as_slice());
        HashValue::from_slice(pow_hash.as_slice())
    }

    fn time(&self) -> &dyn TimeService {
        &self.time_service
    }
}
