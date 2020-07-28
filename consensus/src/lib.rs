// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod argon;
mod consensus;
#[cfg(test)]
mod consensus_test;
pub mod dev;
pub mod difficulty;
pub mod dummy;

use crate::argon::ArgonConsensus;
use crate::consensus::Consensus;
use crate::dev::DevConsensus;
use crate::dummy::DummyConsensus;
use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_traits::ChainReader;
use starcoin_types::block::{Block, BlockHeader, BlockTemplate};
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

pub fn calculate_next_difficulty(
    strategy: ConsensusStrategy,
    reader: &dyn ChainReader,
) -> Result<U256> {
    match strategy {
        ConsensusStrategy::Dummy => DummyConsensus::calculate_next_difficulty(reader),
        ConsensusStrategy::Dev => DevConsensus::calculate_next_difficulty(reader),
        ConsensusStrategy::Argon => ArgonConsensus::calculate_next_difficulty(reader),
    }
}

/// Calculate new block consensus header
// TODO use &HashValue to replace &[u8] for header_hash
pub fn solve_consensus_nonce(
    strategy: ConsensusStrategy,
    header_hash: &[u8],
    difficulty: U256,
) -> u64 {
    match strategy {
        ConsensusStrategy::Dummy => DummyConsensus::solve_consensus_nonce(header_hash, difficulty),
        ConsensusStrategy::Dev => DevConsensus::solve_consensus_nonce(header_hash, difficulty),
        ConsensusStrategy::Argon => ArgonConsensus::solve_consensus_nonce(header_hash, difficulty),
    }
}

pub fn verify(
    strategy: ConsensusStrategy,
    reader: &dyn ChainReader,
    header: &BlockHeader,
) -> Result<()> {
    match strategy {
        ConsensusStrategy::Dummy => DummyConsensus::verify(reader, header),
        ConsensusStrategy::Dev => DevConsensus::verify(reader, header),
        ConsensusStrategy::Argon => ArgonConsensus::verify(reader, header),
    }
}

/// Construct block with BlockTemplate, this a shortcut method for calculate_next_difficulty + solve_consensus_nonce
pub fn create_block(
    strategy: ConsensusStrategy,
    reader: &dyn ChainReader,
    block_template: BlockTemplate,
) -> Result<Block> {
    let difficulty = calculate_next_difficulty(strategy, reader)?;
    let raw_hash = block_template.as_raw_block_header(difficulty).crypto_hash();
    let consensus_nonce = solve_consensus_nonce(strategy, raw_hash.to_vec().as_slice(), difficulty);
    Ok(block_template.into_block(consensus_nonce, difficulty))
}
