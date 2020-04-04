// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use config::NodeConfig;
use futures::channel::oneshot;
use std::convert::TryFrom;
use std::sync::Arc;
use traits::ChainReader;
use types::block::{Block, BlockHeader, BlockTemplate};
use types::U256;

pub mod argon_consensus;
pub mod difficult;
pub mod dummy;

pub trait ConsensusHeader:
    TryFrom<Vec<u8>> + Into<Vec<u8>> + std::marker::Unpin + Clone + Sync + Send
{
}

//TODO merge Consensus and ConsensusHeader to One trait by Trait Associated type.

pub trait Consensus: std::marker::Unpin + Clone + Sync + Send {
    type ConsensusHeader;

    fn init_genesis_header(config: Arc<NodeConfig>) -> (Vec<u8>, U256);

    fn calculate_next_difficulty(reader: &dyn ChainReader) -> U256 {
        difficult::get_next_work_required(reader)
    }

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
