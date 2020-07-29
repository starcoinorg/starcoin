// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ChainReader;
use anyhow::Result;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_state_api::AccountStateReader;
use starcoin_types::{
    block::{Block, BlockHeader, BlockTemplate},
    U256,
};
use starcoin_vm_types::on_chain_config::EpochInfo;

pub trait Consensus: std::marker::Unpin + Clone + Sync + Send {
    fn epoch(chain: &dyn ChainReader) -> Result<EpochInfo> {
        let account_reader = AccountStateReader::new(chain.chain_state_reader());
        account_reader.epoch()
    }

    fn calculate_next_difficulty(reader: &dyn ChainReader) -> Result<U256>;

    /// Calculate new block consensus header
    // TODO use &HashValue to replace &[u8] for header_hash
    fn solve_consensus_nonce(header_hash: &[u8], difficulty: U256) -> u64;

    fn verify(reader: &dyn ChainReader, header: &BlockHeader) -> Result<()>;

    /// Construct block with BlockTemplate, this a shortcut method for calculate_next_difficulty + solve_consensus_nonce
    fn create_block(reader: &dyn ChainReader, block_template: BlockTemplate) -> Result<Block> {
        let difficulty = Self::calculate_next_difficulty(reader)?;
        let raw_hash = block_template.as_raw_block_header(difficulty).crypto_hash();
        let consensus_nonce = Self::solve_consensus_nonce(raw_hash.to_vec().as_slice(), difficulty);
        Ok(block_template.into_block(consensus_nonce, difficulty))
    }
}
