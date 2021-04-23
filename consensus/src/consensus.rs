// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{difficult_to_target, generate_nonce, ChainReader};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::{
    block::{Block, BlockHeader, BlockTemplate},
    U256,
};
use starcoin_vm_types::time::TimeService;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConsensusVerifyError {
    #[error("Verify Difficulty Error, expect: {expect}, got: {real}")]
    VerifyDifficultyError { expect: U256, real: U256 },
    #[error(
    "Verify Nonce Error, expect target: {target}, got: {real}, nonce: {nonce}, extra: {extra}, diff: {diff}"
    )]
    VerifyNonceError {
        target: U256,
        real: U256,
        nonce: u32,
        extra: BlockHeaderExtra,
        diff: U256,
    },
}

pub trait Consensus {
    fn calculate_next_difficulty(&self, reader: &dyn ChainReader) -> Result<U256>;

    /// Calculate new block consensus header
    fn solve_consensus_nonce(
        &self,
        mining_hash: &[u8],
        difficulty: U256,
        _time_service: &dyn TimeService,
    ) -> u32 {
        let mut nonce = generate_nonce();
        let extra = BlockHeaderExtra::new([0u8; 4]);
        loop {
            let pow_hash: U256 = self
                .calculate_pow_hash(mining_hash, nonce, &extra)
                .expect("calculate hash should work")
                .into();
            let target = difficult_to_target(difficulty);
            if pow_hash > target {
                nonce = nonce.saturating_add(1);
                continue;
            }
            break;
        }
        nonce
    }

    fn verify(&self, reader: &dyn ChainReader, header: &BlockHeader) -> Result<()> {
        let difficulty = self.calculate_next_difficulty(reader)?;
        self.verify_header_difficulty(difficulty, header)
    }

    /// Calculate the Pow hash for header
    fn calculate_pow_hash(
        &self,
        pow_header_blob: &[u8],
        nonce: u32,
        extra: &BlockHeaderExtra,
    ) -> Result<HashValue>;

    /// Construct block with BlockTemplate, this a shortcut method for calculate_next_difficulty + solve_consensus_nonce
    fn create_block(
        &self,
        block_template: BlockTemplate,
        time_service: &dyn TimeService,
    ) -> Result<Block> {
        let mining_hash = block_template.as_pow_header_blob();
        let consensus_nonce =
            self.solve_consensus_nonce(&mining_hash, block_template.difficulty, time_service);
        let extra = BlockHeaderExtra::new([0u8; 4]);
        Ok(block_template.into_block(consensus_nonce, extra))
    }
    /// Inner helper for verify and unit testing
    fn verify_header_difficulty(&self, difficulty: U256, header: &BlockHeader) -> Result<()> {
        debug!(
            "verify_header_difficulty, calculate target:{}, header target: {}, nonce: {}",
            difficulty,
            header.difficulty(),
            header.nonce()
        );
        if header.difficulty() != difficulty {
            return Err(ConsensusVerifyError::VerifyDifficultyError {
                expect: difficulty,
                real: header.difficulty(),
            }
            .into());
        }
        let nonce = header.nonce();
        let extra = header.extra();
        let pow_header_blob = header.as_pow_header_blob();
        let pow_hash: U256 = self
            .calculate_pow_hash(&pow_header_blob, nonce, extra)?
            .into();
        let target = difficult_to_target(difficulty);
        if pow_hash > target {
            return Err(ConsensusVerifyError::VerifyNonceError {
                target,
                real: pow_hash,
                nonce,
                extra: *extra,
                diff: difficulty,
            }
            .into());
        }
        Ok(())
    }

    fn verify_blob(
        &self,
        blob: Vec<u8>,
        nonce: u32,
        extra: BlockHeaderExtra,
        difficulty: U256,
    ) -> Result<()> {
        let pow_hash: U256 = self.calculate_pow_hash(&blob, nonce, &extra)?.into();
        let target = difficult_to_target(difficulty);
        if pow_hash > target {
            return Err(ConsensusVerifyError::VerifyNonceError {
                target,
                real: pow_hash,
                nonce,
                extra,
                diff: difficulty,
            }
            .into());
        }
        Ok(())
    }
}
