// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::difficult::difficult_1_target;
use crate::{Consensus, ConsensusHeader};
use anyhow::{Error, Result};
use argon2::{self, Config};
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use config::NodeConfig;
use futures::channel::oneshot::Receiver;
use rand::Rng;
use std::convert::TryFrom;
use std::sync::Arc;
use traits::ChainReader;
use types::block::{Block, BlockHeader, BlockTemplate};
use types::{H256, U256};

#[derive(Clone, Debug)]
pub struct ArgonConsensusHeader {
    nonce: u64,
}

impl ConsensusHeader for ArgonConsensusHeader {}

impl TryFrom<Vec<u8>> for ArgonConsensusHeader {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        Ok(ArgonConsensusHeader {
            nonce: vec_to_u64(value),
        })
    }
}

impl Into<Vec<u8>> for ArgonConsensusHeader {
    fn into(self) -> Vec<u8> {
        u64_to_vec(self.nonce)
    }
}

#[derive(Clone)]
pub struct ArgonConsensus {}

impl Consensus for ArgonConsensus {
    type ConsensusHeader = ArgonConsensusHeader;

    fn init_genesis_header(_config: Arc<NodeConfig>) -> (Vec<u8>, U256) {
        (vec![], difficult_1_target())
    }

    fn solve_consensus_header(header_hash: &[u8], difficulty: U256) -> Self::ConsensusHeader {
        let mut nonce = generate_nonce();
        loop {
            let pow_hash: U256 = calculate_hash(&set_header_nonce(&header_hash, nonce)).into();
            if pow_hash > difficulty {
                nonce += 1;
                continue;
            }
            break;
        }
        ArgonConsensusHeader { nonce }
    }

    fn verify_header(
        _config: Arc<NodeConfig>,
        _reader: &dyn ChainReader,
        header: &BlockHeader,
    ) -> Result<()> {
        let df = header.difficult();
        let nonce = vec_to_u64(Vec::from(header.consensus_header()));
        let header = header.id().to_vec();
        if verify(&header, nonce, df) == true {
            Ok(())
        } else {
            Err(anyhow::Error::msg("invalid header"))
        }
    }

    fn create_block(
        _config: Arc<NodeConfig>,
        _reader: &dyn ChainReader,
        _block_template: BlockTemplate,
        _cancel: Receiver<()>,
    ) -> Result<Block, Error> {
        unimplemented!()
    }
}

pub fn u64_to_vec(u: u64) -> Vec<u8> {
    let mut wtr = vec![];
    wtr.write_u64::<LittleEndian>(u).unwrap();
    wtr
}

fn verify(header: &[u8], nonce: u64, difficulty: U256) -> bool {
    let pow_header = set_header_nonce(header, nonce);
    let pow_hash = calculate_hash(&pow_header);
    let hash_u256: U256 = pow_hash.into();
    if hash_u256 <= difficulty {
        return true;
    }
    return false;
}

pub fn calculate_hash(header: &[u8]) -> H256 {
    let config = Config::default();
    let output = argon2::hash_raw(header, header, &config).unwrap();
    let h_256: H256 = output.as_slice().into();
    h_256
}

fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>();
    rng.gen_range(0, u64::max_value())
}

pub fn set_header_nonce(header: &[u8], nonce: u64) -> Vec<u8> {
    let len = header.len();
    let mut header = header.to_owned();
    header.truncate(len - 8);
    let _ = header.write_u64::<LittleEndian>(nonce);
    header
}

pub fn vec_to_u64(v: Vec<u8>) -> u64 {
    LittleEndian::read_u64(&v)
}
