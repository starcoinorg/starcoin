// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::time::{RealTimeService, TimeService};
use crate::{difficult_to_target, difficulty, set_header_nonce, target_to_difficulty};
use anyhow::{anyhow, Result};
use argon2::{self, Config};
use rand::Rng;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_traits::ChainReader;
use starcoin_types::block::{BlockHeader, RawBlockHeader};
use starcoin_types::{H256, U256};
use starcoin_vm_types::on_chain_config::EpochInfo;

#[derive(Default)]
pub struct ArgonConsensus {
    time_service: RealTimeService,
}

impl ArgonConsensus {
    pub fn new() -> Self {
        Self {
            time_service: RealTimeService::new(),
        }
    }
}

impl Consensus for ArgonConsensus {
    fn calculate_next_difficulty(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
    ) -> Result<U256> {
        let target = difficulty::get_next_work_required(reader, epoch)?;
        Ok(target_to_difficulty(target))
    }
    /// Only for unit testing
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
        //TODO: check mining_hash for difficulty? not need recalculate it?
        let difficulty = self.calculate_next_difficulty(reader, epoch)?;
        if header.difficulty() != difficulty {
            return Err(anyhow!(
                "Difficulty mismatch: {:?}, header: {:?}",
                difficulty,
                header
            ));
        }
        let nonce = header.nonce;
        let raw_block_header: RawBlockHeader = header.to_owned().into();
        let pow_hash = self.calculate_pow_hash(raw_block_header.crypto_hash(), nonce)?;
        let hash_u256: U256 = pow_hash.into();
        let target = difficult_to_target(difficulty);
        if hash_u256 <= target {
            anyhow::bail!("Invalid header:{:?}", header);
        }
        Ok(())
    }

    fn calculate_pow_hash(&self, mining_hash: HashValue, nonce: u64) -> Result<H256> {
        let mix_hash = set_header_nonce(&mining_hash.to_vec(), nonce);
        let mut config = Config::default();
        config.mem_cost = 1024;
        let output = argon2::hash_raw(&mix_hash, &mix_hash, &config)?;
        let h_256: H256 = output.as_slice().into();
        Ok(h_256)
    }

    fn time(&self) -> &dyn TimeService {
        &self.time_service
    }
}

fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>();
    rng.gen_range(0, u64::max_value())
}
