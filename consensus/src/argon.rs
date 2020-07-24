// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::difficulty::{difficult_to_target, target_to_difficulty};
use crate::{difficulty, set_header_nonce};
use anyhow::{anyhow, Result};
use argon2::{self, Config};
use byteorder::{ByteOrder, LittleEndian};
use crypto::hash::PlainCryptoHash;
use logger::prelude::*;
use rand::Rng;
use traits::{ChainReader, Consensus};
use types::block::{BlockHeader, RawBlockHeader};
use types::{H256, U256};

#[derive(Clone)]
pub struct ArgonConsensus {}

impl Consensus for ArgonConsensus {
    fn calculate_next_difficulty(reader: &dyn ChainReader) -> Result<U256> {
        let target = difficulty::get_next_work_required(reader)?;
        Ok(target_to_difficulty(target))
    }

    fn solve_consensus_nonce(header_hash: &[u8], difficulty: U256) -> u64 {
        let mut nonce = generate_nonce();
        loop {
            let pow_hash: U256 = calculate_hash(&set_header_nonce(&header_hash, nonce))
                .expect("calculate hash should work")
                .into();
            let target = difficult_to_target(difficulty);
            if pow_hash > target {
                nonce += 1;
                continue;
            }
            break;
        }
        nonce
    }

    fn verify(reader: &dyn ChainReader, header: &BlockHeader) -> Result<()> {
        let difficulty = ArgonConsensus::calculate_next_difficulty(reader)?;
        if header.difficulty() != difficulty {
            return Err(anyhow!(
                "Difficulty mismatch: {:?}, {:?}",
                header.difficulty(),
                difficulty
            ));
        }
        let nonce = header.nonce;
        debug!(
            "Verify header, nonce, difficulty :{:?}, {:o}, {:x}",
            header, nonce, difficulty
        );
        let raw_block_header: RawBlockHeader = header.clone().into();
        if verify(&raw_block_header.crypto_hash().to_vec(), nonce, difficulty) {
            Ok(())
        } else {
            Err(anyhow::Error::msg("Invalid header"))
        }
    }
}

pub fn verify(header: &[u8], nonce: u64, difficulty: U256) -> bool {
    let pow_header = set_header_nonce(header, nonce);
    let pow_hash = calculate_hash(&pow_header);
    if pow_hash.is_err() {
        return false;
    }
    let hash_u256: U256 = pow_hash.unwrap().into();
    let target = difficult_to_target(difficulty);
    if hash_u256 <= target {
        return true;
    }
    false
}

pub fn calculate_hash(header: &[u8]) -> Result<H256> {
    let mut config = Config::default();
    config.mem_cost = 1024;
    let output = argon2::hash_raw(header, header, &config)?;
    let h_256: H256 = output.as_slice().into();
    Ok(h_256)
}

fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>();
    rng.gen_range(0, u64::max_value())
}

pub fn vec_to_u64(v: Vec<u8>) -> u64 {
    LittleEndian::read_u64(&v)
}
