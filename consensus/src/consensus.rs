// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::time::TimeService;
use crate::{difficult_to_target, ChainReader};
use anyhow::{anyhow, Result};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_state_api::AccountStateReader;
use starcoin_statedb::ChainStateReader;
use starcoin_types::block::RawBlockHeader;
use starcoin_types::{
    block::{Block, BlockHeader, BlockTemplate},
    U256,
};
use starcoin_vm_types::on_chain_config::EpochInfo;

pub trait Consensus {
    fn epoch(chain: &dyn ChainReader) -> Result<EpochInfo> {
        let account_reader = AccountStateReader::new(chain.chain_state_reader());
        account_reader.get_epoch_info()
    }

    /// Init consensus with on chain state
    fn init(&self, _reader: &dyn ChainStateReader) -> Result<()> {
        Ok(())
    }

    fn calculate_next_difficulty(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
    ) -> Result<U256>;

    /// Calculate new block consensus header
    fn solve_consensus_nonce(&self, mining_hash: HashValue, difficulty: U256) -> u64;

    fn verify(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
        header: &BlockHeader,
    ) -> Result<()>;

    /// Calculate the Pow hash for header
    fn calculate_pow_hash(&self, mining_hash: HashValue, nonce: u64) -> Result<HashValue>;

    /// Construct block with BlockTemplate, this a shortcut method for calculate_next_difficulty + solve_consensus_nonce
    fn create_block(
        &self,
        reader: &dyn ChainReader,
        block_template: BlockTemplate,
    ) -> Result<Block> {
        let epoch = Self::epoch(reader)?;
        let difficulty = self.calculate_next_difficulty(reader, &epoch)?;
        let mining_hash = block_template.as_raw_block_header(difficulty).crypto_hash();
        let consensus_nonce = self.solve_consensus_nonce(mining_hash, difficulty);
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
        let raw_block_header: RawBlockHeader = header.to_owned().into();
        let pow_hash: U256 = self
            .calculate_pow_hash(raw_block_header.crypto_hash(), nonce)?
            .into();
        let target = difficult_to_target(difficulty);
        if pow_hash > target {
            anyhow::bail!("Invalid header:{:?}", header);
        }
        Ok(())
    }
}
