// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Consensus, ConsensusHeader};
use anyhow::{Error, Result};
use futures::channel::oneshot::Receiver;
use std::convert::TryFrom;
use traits::ChainReader;
use types::block::{Block, BlockHeader, BlockTemplate};

pub struct DummyHeader {}

impl ConsensusHeader for DummyHeader {}

impl TryFrom<Vec<u8>> for DummyHeader {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        Ok(DummyHeader {})
    }
}

impl Into<Vec<u8>> for DummyHeader {
    fn into(self) -> Vec<u8> {
        vec![]
    }
}

pub struct DummyConsensus {}

impl Consensus for DummyConsensus {
    fn verify_header(reader: &dyn ChainReader, header: &BlockHeader) -> Result<()> {
        Ok(())
    }

    fn create_block(
        reader: &dyn ChainReader,
        block_template: BlockTemplate,
        cancel: Receiver<()>,
    ) -> Result<Block> {
        Ok(block_template.into_block(DummyHeader {}))
    }
}
