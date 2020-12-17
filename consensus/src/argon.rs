// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::{difficulty, set_header_nonce, target_to_difficulty};
use anyhow::Result;
use argon2::{self, Config};
use starcoin_crypto::HashValue;
use starcoin_traits::ChainReader;
use starcoin_types::U256;

#[derive(Default)]
pub struct ArgonConsensus {}

impl ArgonConsensus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Consensus for ArgonConsensus {
    fn calculate_next_difficulty(&self, reader: &dyn ChainReader) -> Result<U256> {
        let target = difficulty::get_next_work_required(reader)?;
        Ok(target_to_difficulty(target))
    }

    fn calculate_pow_hash(&self, mining_hash: &[u8], nonce: u32) -> Result<HashValue> {
        let mix_hash = set_header_nonce(mining_hash, nonce);
        let mut config = Config::default();
        config.mem_cost = 1024;
        let output = argon2::hash_raw(&mix_hash, &mix_hash, &config)?;
        HashValue::from_slice(output.as_slice())
    }
}
