// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::{difficulty, set_header_nonce, target_to_difficulty};
use anyhow::Result;
use sha3::{Digest, Keccak256};
use starcoin_crypto::HashValue;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::on_chain_resource::EpochInfo;

#[derive(Default)]
pub struct KeccakConsensus {}

impl KeccakConsensus {
    pub fn new() -> Self {
        Self {}
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
    fn calculate_pow_hash(&self, mining_hash: &[u8], nonce: u32) -> Result<HashValue> {
        let mix_hash = set_header_nonce(mining_hash, nonce);
        let pow_hash = Keccak256::digest(Keccak256::digest(&mix_hash).as_slice());
        HashValue::from_slice(pow_hash.as_slice())
    }
}
