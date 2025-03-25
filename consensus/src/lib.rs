// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::arithmetic_side_effects)]
use crate::argon::ArgonConsensus;
use crate::cn::CryptoNightConsensus;
use crate::dummy::DummyConsensus;
use crate::keccak::KeccakConsensus;
use anyhow::{format_err, Result};
use once_cell::sync::Lazy;
use rand::Rng;
use starcoin_chain_api::ChainReader;
use starcoin_crypto::HashValue;
use starcoin_time_service::TimeService;
use starcoin_types::block::{BlockHeader, BlockHeaderExtra};
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::ConsensusStrategy;

pub mod argon;
pub mod cn;
mod consensus;
#[cfg(test)]
mod consensus_test;
pub mod difficulty;
pub mod dummy;
pub mod keccak;

pub use consensus::{Consensus, ConsensusVerifyError};
pub use starcoin_time_service::duration_since_epoch;

pub fn target_to_difficulty(target: U256) -> Result<U256> {
    U256::MAX
        .checked_div(target)
        .ok_or(format_err!("zero-divisor"))
}

pub fn difficult_to_target(difficulty: U256) -> Result<U256> {
    U256::MAX
        .checked_div(difficulty)
        .ok_or(format_err!("zero-divisor"))
}

pub fn set_header_nonce(header: &[u8], nonce: u32, extra: &BlockHeaderExtra) -> Vec<u8> {
    let len = header.len();
    if len != 76 {
        return vec![];
    }
    let mut header = header.to_owned();
    header[35..39].copy_from_slice(extra.as_slice());
    header[39..43].copy_from_slice(&nonce.to_le_bytes());
    header
}

static G_DUMMY: Lazy<DummyConsensus> = Lazy::new(DummyConsensus::new);
static G_ARGON: Lazy<ArgonConsensus> = Lazy::new(ArgonConsensus::new);
static G_KECCAK: Lazy<KeccakConsensus> = Lazy::new(KeccakConsensus::new);
pub static G_CRYPTONIGHT: Lazy<CryptoNightConsensus> = Lazy::new(CryptoNightConsensus::new);

impl Consensus for ConsensusStrategy {
    fn calculate_next_difficulty(&self, reader: &dyn ChainReader) -> Result<U256> {
        match self {
            Self::Dummy => G_DUMMY.calculate_next_difficulty(reader),
            Self::Argon => G_ARGON.calculate_next_difficulty(reader),
            Self::Keccak => G_KECCAK.calculate_next_difficulty(reader),
            Self::CryptoNight => G_CRYPTONIGHT.calculate_next_difficulty(reader),
        }
    }

    fn solve_consensus_nonce(
        &self,
        mining_hash: &[u8],
        difficulty: U256,
        time_service: &dyn TimeService,
    ) -> u32 {
        match self {
            Self::Dummy => G_DUMMY.solve_consensus_nonce(mining_hash, difficulty, time_service),
            Self::Argon => G_ARGON.solve_consensus_nonce(mining_hash, difficulty, time_service),
            Self::Keccak => G_KECCAK.solve_consensus_nonce(mining_hash, difficulty, time_service),
            Self::CryptoNight => {
                G_CRYPTONIGHT.solve_consensus_nonce(mining_hash, difficulty, time_service)
            }
        }
    }

    fn verify(&self, reader: &dyn ChainReader, header: &BlockHeader) -> Result<()> {
        match self {
            Self::Dummy => G_DUMMY.verify(reader, header),
            Self::Argon => G_ARGON.verify(reader, header),
            Self::Keccak => G_KECCAK.verify(reader, header),
            Self::CryptoNight => G_CRYPTONIGHT.verify(reader, header),
        }
    }

    fn calculate_pow_hash(
        &self,
        mining_hash: &[u8],
        nonce: u32,
        extra: &BlockHeaderExtra,
    ) -> Result<HashValue> {
        match self {
            Self::Dummy => G_DUMMY.calculate_pow_hash(mining_hash, nonce, extra),
            Self::Argon => G_ARGON.calculate_pow_hash(mining_hash, nonce, extra),
            Self::Keccak => G_KECCAK.calculate_pow_hash(mining_hash, nonce, extra),
            Self::CryptoNight => G_CRYPTONIGHT.calculate_pow_hash(mining_hash, nonce, extra),
        }
    }
}

pub fn generate_nonce() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>();
    rng.gen_range(0..u32::MAX)
}
