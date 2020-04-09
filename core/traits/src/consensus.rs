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

    fn calculate_next_difficulty(config: Arc<NodeConfig>, reader: &dyn ChainReader) -> U256;

    fn solve_consensus_header(pow_hash: &[u8], difficulty: U256) -> Self::ConsensusHeader;

    fn verify_header(
        config: Arc<NodeConfig>,
        reader: &dyn ChainReader,
        header: &BlockHeader,
    ) -> Result<()>;
    /// Construct block with BlockTemplate, Only for test
    fn create_block(
        config: Arc<NodeConfig>,
        reader: &dyn ChainReader,
        block_template: BlockTemplate,
    ) -> Result<Block>;
}
