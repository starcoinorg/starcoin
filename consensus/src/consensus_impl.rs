// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Consensus, ConsensusHeader};
use anyhow::{Error, Result};
use futures::channel::oneshot::Receiver;
use std::convert::TryFrom;
use traits::ChainReader;
use types::block::{Block, BlockHeader, BlockTemplate};
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};

pub struct ConsensusHeaderImpl {
    nonce: u64,
}

impl ConsensusHeader for ConsensusHeaderImpl {}

impl TryFrom<Vec<u8>> for ConsensusHeaderImpl {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        Ok(ConsensusHeaderImpl { nonce: vec_to_u64(value) })
    }
}

impl Into<Vec<u8>> for ConsensusHeaderImpl {
    fn into(self) -> Vec<u8> {
        u64_to_vec(self.nonce)
    }
}

pub struct ConsensusImpl {}

impl Consensus for ConsensusImpl {
    fn verify_header(reader: &dyn ChainReader, header: &BlockHeader) -> Result<()> {
        Ok(())
    }
}


pub fn u64_to_vec(u: u64) -> Vec<u8> {
    let mut wtr = vec![];
    wtr.write_u64::<LittleEndian>(u).unwrap();
    wtr
}

pub fn vec_to_u64(v: Vec<u8>) -> u64 {
    LittleEndian::read_u64(&v)
}