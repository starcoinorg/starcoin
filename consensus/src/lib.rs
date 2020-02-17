// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use config::NodeConfig;
use crypto::HashValue;
use std::convert::TryFrom;
use types::block::{Block, BlockHeader, BlockNumber};

pub mod dummy;

pub trait ConsensusHeader: TryFrom<Vec<u8>> + Into<Vec<u8>> {}

pub trait ChainReader {
    fn current_header(&self) -> BlockHeader;
    fn get_header(&self, hash: HashValue) -> BlockHeader;
    fn get_header_by_number(&self, number: BlockNumber) -> BlockHeader;
    fn get_block(&self, hash: HashValue) -> Block;
}

pub trait Consensus<H>
where
    H: ConsensusHeader,
{
    fn verify_header(reader: &dyn ChainReader, header: &BlockHeader) -> Result<()>;
    fn create_header(reader: &dyn ChainReader) -> Result<H>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
