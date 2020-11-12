// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::difficulty::{get_next_target_helper, BlockDiffInfo};
use crate::{difficult_to_target, target_to_difficulty, CRYPTONIGHT};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_types::block::{BlockHeader, RawBlockHeader};
use starcoin_types::U256;
use starcoin_vm_types::time::{
    duration_since_epoch, MockTimeService, TimeService, TimeServiceType,
};
use std::collections::VecDeque;

#[stest::test]
fn raw_hash_test() {
    let mut header = BlockHeader::random();
    let header_1 = header.clone();
    let id_1 = header_1.id();
    let raw_1: RawBlockHeader = header_1.into();
    let raw_id_1 = raw_1.crypto_hash();
    header.nonce = 42;
    let header_2 = header;
    let id_2 = header_2.id();
    let raw_2: RawBlockHeader = header_2.into();
    let raw_id_2 = raw_2.crypto_hash();
    assert_ne!(id_1, id_2);
    assert_eq!(raw_id_1, raw_id_2);
}

#[stest::test]
fn verify_header_test() {
    let mut header = BlockHeader::random();
    header.difficulty = 1.into();
    let raw_header: RawBlockHeader = header.clone().into();
    let time_service = TimeServiceType::RealTimeService.new_time_service();
    let nonce = CRYPTONIGHT.solve_consensus_nonce(
        &header.as_pow_header_blob(),
        raw_header.difficulty,
        time_service.as_ref(),
    );
    header.nonce = nonce;
    CRYPTONIGHT
        .verify_header_difficulty(header.difficulty, &header)
        .unwrap()
}

#[stest::test]
fn test_get_next_target() {
    let time_used = simulate_blocks(15, 10000.into());
    assert!((time_used as i64 - 15).abs() <= 1);
    let time_used = simulate_blocks(20, 20000.into());
    assert!((time_used as i64 - 20).abs() <= 1);
    let time_used = simulate_blocks(5, 1000.into());
    assert!((time_used as i64 - 5).abs() <= 1);
}

fn simulate_blocks(time_plan: u64, init_difficulty: U256) -> u64 {
    fn liner_hash_pow(difficulty: U256, current: u64) -> u64 {
        let ts = MockTimeService::new_with_value(current);
        let used_time = difficulty.as_u64() / 10;
        ts.sleep(used_time);
        ts.now_millis()
    }

    let mut diff = init_difficulty;
    let mut blocks = VecDeque::new();
    let mut now = duration_since_epoch().as_millis() as u64;
    for _ in 0..500 {
        let timestamp = liner_hash_pow(diff, now);
        now = timestamp;
        blocks.push_front(BlockDiffInfo::new(timestamp, difficult_to_target(diff)));
        let bf: Vec<&BlockDiffInfo> = blocks.iter().collect();
        let blocks = bf.iter().map(|&b| b.clone()).collect();
        diff = target_to_difficulty(get_next_target_helper(blocks, time_plan).unwrap());
    }
    blocks[0].timestamp - blocks[1].timestamp
}
