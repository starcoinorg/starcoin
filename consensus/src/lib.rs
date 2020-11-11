// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod argon;
pub mod cn;
mod consensus;
#[cfg(test)]
mod consensus_test;
pub mod difficulty;
pub mod dummy;
pub mod keccak;

pub use consensus::Consensus;
pub use starcoin_vm_types::time::duration_since_epoch;

use crate::argon::ArgonConsensus;
use crate::cn::CryptoNightConsensus;
use crate::dummy::DummyConsensus;
use crate::keccak::KeccakConsensus;
use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use once_cell::sync::Lazy;
use rand::Rng;
use starcoin_crypto::HashValue;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use starcoin_vm_types::on_chain_resource::EpochInfo;
use starcoin_vm_types::time::TimeService;

pub fn target_to_difficulty(target: U256) -> U256 {
    U256::max_value() / target
}

pub fn difficult_to_target(difficulty: U256) -> U256 {
    U256::max_value() / difficulty
}

pub fn set_header_nonce(header: &[u8], nonce: u32) -> Vec<u8> {
    let len = header.len();
    if len != 76 {
        return vec![];
    }
    let mut header = header.to_owned();

    let _ = header[39..].as_mut().write_u32::<LittleEndian>(nonce);
    header
}

static DUMMY: Lazy<DummyConsensus> = Lazy::new(DummyConsensus::new);
static ARGON: Lazy<ArgonConsensus> = Lazy::new(ArgonConsensus::new);
static KECCAK: Lazy<KeccakConsensus> = Lazy::new(KeccakConsensus::new);
pub static CRYPTONIGHT: Lazy<CryptoNightConsensus> = Lazy::new(CryptoNightConsensus::new);

impl Consensus for ConsensusStrategy {
    fn calculate_next_difficulty(
        &self,
        reader: &dyn ChainReader,
        epoch: &EpochInfo,
    ) -> Result<U256> {
        match self {
            ConsensusStrategy::Dummy => DUMMY.calculate_next_difficulty(reader, epoch),
            ConsensusStrategy::Argon => ARGON.calculate_next_difficulty(reader, epoch),
            ConsensusStrategy::Keccak => KECCAK.calculate_next_difficulty(reader, epoch),
            ConsensusStrategy::CryptoNight => CRYPTONIGHT.calculate_next_difficulty(reader, epoch),
        }
    }

    fn solve_consensus_nonce(
        &self,
        mining_hash: &[u8],
        difficulty: U256,
        time_service: &dyn TimeService,
    ) -> u32 {
        match self {
            ConsensusStrategy::Dummy => {
                DUMMY.solve_consensus_nonce(mining_hash, difficulty, time_service)
            }
            ConsensusStrategy::Argon => {
                ARGON.solve_consensus_nonce(mining_hash, difficulty, time_service)
            }
            ConsensusStrategy::Keccak => {
                KECCAK.solve_consensus_nonce(mining_hash, difficulty, time_service)
            }
            ConsensusStrategy::CryptoNight => {
                CRYPTONIGHT.solve_consensus_nonce(mining_hash, difficulty, time_service)
            }
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
            ConsensusStrategy::Argon => ARGON.verify(reader, epoch, header),
            ConsensusStrategy::Keccak => KECCAK.verify(reader, epoch, header),
            ConsensusStrategy::CryptoNight => CRYPTONIGHT.verify(reader, epoch, header),
        }
    }

    fn calculate_pow_hash(&self, mining_hash: &[u8], nonce: u32) -> Result<HashValue> {
        match self {
            ConsensusStrategy::Dummy => DUMMY.calculate_pow_hash(mining_hash, nonce),
            ConsensusStrategy::Argon => ARGON.calculate_pow_hash(mining_hash, nonce),
            ConsensusStrategy::Keccak => KECCAK.calculate_pow_hash(mining_hash, nonce),
            ConsensusStrategy::CryptoNight => CRYPTONIGHT.calculate_pow_hash(mining_hash, nonce),
        }
    }
}

pub fn generate_nonce() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>();
    rng.gen_range(0, u32::max_value())
}
