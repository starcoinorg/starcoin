// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::time::{RealTimeService, TimeService};
use crate::{difficult_to_target, difficulty, set_header_nonce, target_to_difficulty};
use anyhow::{anyhow, Result};
use sha3::{Digest, Keccak256};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_traits::ChainReader;
use starcoin_types::block::{BlockHeader, RawBlockHeader};
use starcoin_types::{H256, U256};
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

    fn solve_consensus_nonce(&self, _mining_hash: HashValue, _difficulty: U256) -> u64 {
        unreachable!()
    }

    fn verify(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
        header: &BlockHeader,
    ) -> Result<()> {
        //TODO: check mining_hash for difficulty? not need recalculate it?
        //TODO: Move as a common one.
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
        let pow_hash: H256 = Keccak256::digest(&mix_hash).as_slice().into();
        Ok(pow_hash)
    }

    fn time(&self) -> &dyn TimeService {
        &self.time_service
    }
}
