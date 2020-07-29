// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod argon;
mod consensus;
#[cfg(test)]
mod consensus_test;
pub mod dev;
pub mod difficulty;
pub mod dummy;
mod time;

pub use consensus::Consensus;
pub use time::TimeService;

use crate::argon::ArgonConsensus;
use crate::dev::DevConsensus;
use crate::dummy::DummyConsensus;
use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use once_cell::sync::Lazy;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::chain_config::ConsensusStrategy;

pub fn set_header_nonce(header: &[u8], nonce: u64) -> Vec<u8> {
    let len = header.len();
    let mut header = header.to_owned();
    header.truncate(len - 8);
    let _ = header.write_u64::<LittleEndian>(nonce);
    header
}

pub fn u64_to_vec(u: u64) -> Vec<u8> {
    let mut wtr = vec![];
    wtr.write_u64::<LittleEndian>(u).unwrap();
    wtr
}

static DUMMY: Lazy<DummyConsensus> = Lazy::new(DummyConsensus::new);
static DEV: Lazy<DevConsensus> = Lazy::new(DevConsensus::new);
static ARGON: Lazy<ArgonConsensus> = Lazy::new(ArgonConsensus::new);

impl Consensus for ConsensusStrategy {
    fn calculate_next_difficulty(&self, reader: &dyn ChainReader) -> Result<U256> {
        match self {
            ConsensusStrategy::Dummy => DUMMY.calculate_next_difficulty(reader),
            ConsensusStrategy::Dev => DEV.calculate_next_difficulty(reader),
            ConsensusStrategy::Argon => ARGON.calculate_next_difficulty(reader),
        }
    }

    fn solve_consensus_nonce(&self, header_hash: &[u8], difficulty: U256) -> u64 {
        match self {
            ConsensusStrategy::Dummy => DUMMY.solve_consensus_nonce(header_hash, difficulty),
            ConsensusStrategy::Dev => DEV.solve_consensus_nonce(header_hash, difficulty),
            ConsensusStrategy::Argon => ARGON.solve_consensus_nonce(header_hash, difficulty),
        }
    }

    fn verify(&self, reader: &dyn ChainReader, header: &BlockHeader) -> Result<()> {
        match self {
            ConsensusStrategy::Dummy => DUMMY.verify(reader, header),
            ConsensusStrategy::Dev => DEV.verify(reader, header),
            ConsensusStrategy::Argon => ARGON.verify(reader, header),
        }
    }

    fn time(&self) -> &dyn TimeService {
        match self {
            ConsensusStrategy::Dummy => DUMMY.time(),
            ConsensusStrategy::Dev => DEV.time(),
            ConsensusStrategy::Argon => ARGON.time(),
        }
    }
}
