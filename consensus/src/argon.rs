// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::{difficulty, set_header_nonce, target_to_difficulty};
use anyhow::Result;
use argon2::{self, Config};
use starcoin_chain_api::ChainReader;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockHeaderExtra;
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

    fn calculate_pow_hash(
        &self,
        mining_hash: &[u8],
        nonce: u32,
        extra: &BlockHeaderExtra,
    ) -> Result<HashValue> {
        let mix_hash = set_header_nonce(mining_hash, nonce, extra);
        let config = Config {
            mem_cost: 1024,
            ..Default::default()
        };
        let output = argon2::hash_raw(&mix_hash, &mix_hash, &config)?;
        Ok(HashValue::from_slice(output.as_slice())?)
    }
}
