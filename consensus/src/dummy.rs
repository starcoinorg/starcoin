// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Consensus, ConsensusHeader};
use anyhow::{Error, Result};
use config::NodeConfig;
use futures::channel::oneshot::Receiver;
use logger::prelude::*;
use rand::prelude::*;
use std::convert::TryFrom;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use traits::ChainReader;
use types::block::{Block, BlockHeader, BlockTemplate};

#[derive(Clone, Debug)]
pub struct DummyHeader {}

impl ConsensusHeader for DummyHeader {}

impl TryFrom<Vec<u8>> for DummyHeader {
    type Error = Error;

    fn try_from(_value: Vec<u8>) -> Result<Self> {
        Ok(DummyHeader {})
    }
}

impl Into<Vec<u8>> for DummyHeader {
    fn into(self) -> Vec<u8> {
        vec![]
    }
}

#[derive(Clone)]
pub struct DummyConsensus {}

impl Consensus for DummyConsensus {
    fn init_genesis_header(_config: Arc<NodeConfig>) -> Vec<u8> {
        vec![]
    }

    fn verify_header(
        _config: Arc<NodeConfig>,
        _reader: &dyn ChainReader,
        _header: &BlockHeader,
    ) -> Result<()> {
        Ok(())
    }

    fn create_block(
        _config: Arc<NodeConfig>,
        _reader: &dyn ChainReader,
        block_template: BlockTemplate,
        _cancel: Receiver<()>,
    ) -> Result<Block> {
        Ok(block_template.into_block(DummyHeader {}))
    }
}
