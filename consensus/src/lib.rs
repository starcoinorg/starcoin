// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use config::NodeConfig;
use futures::channel::oneshot;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::sync::Arc;
use traits::ChainReader;
use types::block::{Block, BlockHeader, BlockTemplate};
use types::U256;

pub mod argon_consensus;
pub mod difficult;
pub mod dummy;

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
        cancel: oneshot::Receiver<()>,
    ) -> Result<Block>;
}
