// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::{difficulty, set_header_nonce, target_to_difficulty};
use anyhow::Result;
use sha3::{Digest, Keccak256};
use starcoin_chain_api::ChainReader;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::U256;

#[derive(Default)]
pub struct KeccakConsensus {}

impl KeccakConsensus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Consensus for KeccakConsensus {
    fn calculate_next_difficulty(&self, reader: &dyn ChainReader) -> Result<U256> {
        let target = difficulty::get_next_work_required(reader)?;
        Ok(target_to_difficulty(target))
    }

    /// Double keccak256 for pow hash
    fn calculate_pow_hash(
        &self,
        mining_hash: &[u8],
        nonce: u32,
        extra: &BlockHeaderExtra,
    ) -> Result<HashValue> {
        let mix_hash = set_header_nonce(mining_hash, nonce, extra);
        let pow_hash = Keccak256::digest(Keccak256::digest(&mix_hash).as_slice());
        Ok(HashValue::from_slice(pow_hash.as_slice())?)
    }
}
