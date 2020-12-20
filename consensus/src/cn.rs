// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::{difficulty, set_header_nonce, target_to_difficulty};
use anyhow::Result;
use cryptonight::cryptonight_r;
use starcoin_crypto::HashValue;
use starcoin_traits::ChainReader;
use starcoin_types::U256;

#[derive(Default)]
pub struct CryptoNightConsensus {}

impl CryptoNightConsensus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Consensus for CryptoNightConsensus {
    fn calculate_next_difficulty(&self, reader: &dyn ChainReader) -> Result<U256> {
        let target = difficulty::get_next_work_required(reader)?;
        Ok(target_to_difficulty(target))
    }

    /// CryptoNight-R
    fn calculate_pow_hash(&self, mining_hash: &[u8], nonce: u32) -> Result<HashValue> {
        let mix_hash = set_header_nonce(mining_hash, nonce);
        let pow_hash = cryptonight_r(&mix_hash, mix_hash.len());
        HashValue::from_slice(pow_hash.as_slice())
    }
}
