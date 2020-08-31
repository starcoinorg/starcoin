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
use starcoin_state_api::ChainStateReader;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use starcoin_vm_types::on_chain_config::EpochInfo;

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
    fn init(&self, reader: &dyn ChainStateReader) -> Result<()> {
        match self {
            ConsensusStrategy::Dummy => DUMMY.init(reader),
            ConsensusStrategy::Dev => DEV.init(reader),
            ConsensusStrategy::Argon => ARGON.init(reader),
        }
    }

    fn calculate_next_difficulty(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
    ) -> Result<U256> {
        match self {
            ConsensusStrategy::Dummy => DUMMY.calculate_next_difficulty(reader, epoch),
            ConsensusStrategy::Dev => DEV.calculate_next_difficulty(reader, epoch),
            ConsensusStrategy::Argon => ARGON.calculate_next_difficulty(reader, epoch),
        }
    }

    fn solve_consensus_nonce(&self, header_hash: &[u8], difficulty: U256) -> u64 {
        match self {
            ConsensusStrategy::Dummy => DUMMY.solve_consensus_nonce(header_hash, difficulty),
            ConsensusStrategy::Dev => DEV.solve_consensus_nonce(header_hash, difficulty),
            ConsensusStrategy::Argon => ARGON.solve_consensus_nonce(header_hash, difficulty),
        }
    }

    fn verify(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
        header: &BlockHeader,
    ) -> Result<()> {
        match self {
            ConsensusStrategy::Dummy => DUMMY.verify(reader, epoch, header),
            ConsensusStrategy::Dev => DEV.verify(reader, epoch, header),
            ConsensusStrategy::Argon => ARGON.verify(reader, epoch, header),
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
