// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ChainReader, Consensus, ConsensusHeader};
use anyhow::{Error, Result};
use std::convert::TryFrom;
use types::block::BlockHeader;

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

impl Consensus<DummyHeader> for DummyConsensus {
    fn verify_header(reader: &dyn ChainReader, header: &BlockHeader) -> Result<()> {
        Ok(())
    }

    fn create_header(reader: &dyn ChainReader) -> Result<DummyHeader> {
        Ok(DummyHeader {})
    }
}
