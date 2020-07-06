// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ChainReader;
use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_types::{
    block::{Block, BlockHeader, BlockTemplate},
    U256,
};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::sync::Arc;

pub trait ConsensusHeader:
    TryFrom<Vec<u8>> + Into<Vec<u8>> + std::marker::Unpin + Clone + Sync + Send + Debug
{
}

pub trait Consensus: std::marker::Unpin + Clone + Sync + Send {
    type ConsensusHeader: ConsensusHeader;

    fn calculate_next_difficulty(config: Arc<NodeConfig>, reader: &dyn ChainReader)
        -> Result<U256>;

    /// Calculate new block consensus header
    // TODO use &HashValue to replace &[u8] for parent_hash
    fn solve_consensus_header(parent_hash: &[u8], difficulty: U256) -> Self::ConsensusHeader;

    fn verify(
        config: Arc<NodeConfig>,
        reader: &dyn ChainReader,
        header: &BlockHeader,
    ) -> Result<()>;

    /// Construct block with BlockTemplate, this a shortcut method for calculate_next_difficulty + solve_consensus_header
    fn create_block(
        config: Arc<NodeConfig>,
        reader: &dyn ChainReader,
        block_template: BlockTemplate,
    ) -> Result<Block> {
        let difficulty = Self::calculate_next_difficulty(config, reader)?;
        let raw_hash = block_template
            .clone()
            .into_raw_block_header(difficulty)
            .raw_hash();
        let consensus_header =
            Self::solve_consensus_header(raw_hash.to_vec().as_slice(), difficulty);
        Ok(block_template.into_block(consensus_header, difficulty))
    }
}
