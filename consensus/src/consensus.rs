// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::time::TimeService;
use crate::ChainReader;
use anyhow::Result;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_state_api::{AccountStateReader, ChainStateReader};
use starcoin_types::{
    block::{Block, BlockHeader, BlockTemplate},
    U256,
};
use starcoin_vm_types::on_chain_config::EpochInfo;

pub trait Consensus {
    fn epoch(chain: &dyn ChainReader) -> Result<EpochInfo> {
        let account_reader = AccountStateReader::new(chain.chain_state_reader());
        account_reader.epoch()
    }

    fn calculate_next_difficulty(&self, reader: &dyn ChainReader, epoch: &EpochInfo) -> Result<U256>;

    /// Calculate new block consensus header
    fn solve_consensus_nonce(&self, header_hash: &[u8], difficulty: U256) -> u64;

    fn verify(&self, reader: &dyn ChainReader, epoch:&EpochInfo, header: &BlockHeader) -> Result<()>;

    /// Construct block with BlockTemplate, this a shortcut method for calculate_next_difficulty + solve_consensus_nonce
    fn create_block(
        &self,
        reader: &dyn ChainReader,
        block_template: BlockTemplate,
        epoch: &EpochInfo,
    ) -> Result<Block> {
        let difficulty = self.calculate_next_difficulty(reader, epoch)?;
        let raw_hash = block_template.as_raw_block_header(difficulty).crypto_hash();
        let consensus_nonce = self.solve_consensus_nonce(raw_hash.to_vec().as_slice(), difficulty);
        Ok(block_template.into_block(consensus_nonce, difficulty))
    }

    fn time(&self) -> &dyn TimeService;

    fn now(&self) -> u64 {
        self.time().now()
    }
}
