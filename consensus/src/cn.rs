// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::{difficulty, set_header_nonce, target_to_difficulty};
use anyhow::Result;
use cryptonight::cryptonight_r;
use starcoin_crypto::HashValue;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::on_chain_config::EpochInfo;

#[derive(Default)]
pub struct CryptoNightConsensus {}

impl CryptoNightConsensus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Consensus for CryptoNightConsensus {
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

    /// CryptoNight-R
    fn calculate_pow_hash(&self, mining_hash: HashValue, nonce: u64) -> Result<HashValue> {
        let mix_hash = set_header_nonce(&mining_hash.to_vec(), nonce);
        let pow_hash = cryptonight_r(&mix_hash, mix_hash.len());
        HashValue::from_slice(pow_hash.as_slice())
    }
}
