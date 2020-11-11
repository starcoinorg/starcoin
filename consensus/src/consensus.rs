// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{difficult_to_target, generate_nonce, ChainReader};
use anyhow::{anyhow, Result};
use starcoin_crypto::HashValue;
use starcoin_types::{
    block::{Block, BlockHeader, BlockTemplate},
    U256,
};
use starcoin_vm_types::on_chain_resource::EpochInfo;
use starcoin_vm_types::time::TimeService;

pub trait Consensus {
    fn calculate_next_difficulty(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
    ) -> Result<U256>;

    /// Calculate new block consensus header
    fn solve_consensus_nonce(
        &self,
        mining_hash: &[u8],
        difficulty: U256,
        _time_service: &dyn TimeService,
    ) -> u32 {
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
    ) -> Result<()>;

    /// Calculate the Pow hash for header
    fn calculate_pow_hash(&self, pow_header_blob: &[u8], nonce: u32) -> Result<HashValue>;

    /// Construct block with BlockTemplate, this a shortcut method for calculate_next_difficulty + solve_consensus_nonce
    fn create_block(
        &self,
        reader: &dyn ChainReader,
        block_template: BlockTemplate,
        time_service: &dyn TimeService,
    ) -> Result<Block> {
        let epoch = reader.epoch_info()?;
        let difficulty = self.calculate_next_difficulty(reader, &epoch)?;
        let mining_hash = block_template.as_pow_header_blob(difficulty);
        let consensus_nonce = self.solve_consensus_nonce(&mining_hash, difficulty, time_service);
        Ok(block_template.into_block(consensus_nonce, difficulty))
    }
    /// Inner helper for verify and unit testing
    fn verify_header_difficulty(&self, difficulty: U256, header: &BlockHeader) -> Result<()> {
        if header.difficulty() != difficulty {
            return Err(anyhow!(
                "Difficulty mismatch: {:?}, header: {:?}",
                difficulty,
                header
            ));
        }
        let nonce = header.nonce;
        let pow_header_blob = header.as_pow_header_blob();
        let pow_hash: U256 = self.calculate_pow_hash(&pow_header_blob, nonce)?.into();
        let target = difficult_to_target(difficulty);
        if pow_hash > target {
            anyhow::bail!("Invalid header:{:?}", header);
        }
        Ok(())
    }
}
