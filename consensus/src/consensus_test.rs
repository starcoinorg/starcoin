// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::integer_arithmetic)]

use crate::consensus::Consensus;
use crate::difficulty::{get_next_target_helper, BlockDiffInfo};
use crate::{difficult_to_target, target_to_difficulty, G_CRYPTONIGHT};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_time_service::{duration_since_epoch, MockTimeService, TimeService, TimeServiceType};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{BlockHeader, BlockHeaderBuilder, BlockHeaderExtra, RawBlockHeader};
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::ChainId;
use std::collections::VecDeque;

#[stest::test]
fn raw_hash_test() {
    let header = BlockHeader::random();
    let header_1 = header.clone();
    let id_1 = header_1.id();
    let raw_1: RawBlockHeader = header_1.into();
    let raw_id_1 = raw_1.crypto_hash();
    let header_2 = header.as_builder().with_nonce(42).build();
    let id_2 = header_2.id();
    let raw_2: RawBlockHeader = header_2.into();
    let raw_id_2 = raw_2.crypto_hash();
    assert_ne!(id_1, id_2);
    assert_eq!(raw_id_1, raw_id_2);
}

#[stest::test]
fn verify_header_test() {
    let header = BlockHeaderBuilder::random()
        .with_difficulty(1.into())
        .build();
    let raw_header: RawBlockHeader = header.clone().into();
    let time_service = TimeServiceType::RealTimeService.new_time_service();
    let nonce = G_CRYPTONIGHT.solve_consensus_nonce(
        &header.as_pow_header_blob(),
        raw_header.difficulty,
        time_service.as_ref(),
    );
    let header = header.as_builder().with_nonce(nonce).build();
    G_CRYPTONIGHT
        .verify_header_difficulty(header.difficulty(), &header)
        .unwrap()
}

#[stest::test]
fn test_get_next_target() {
    let time_used = simulate_blocks(15_000, 10000.into());
    assert!((time_used as i64 - 15_000).abs() <= 1000);
    let time_used = simulate_blocks(20_000, 20000.into());
    assert!((time_used as i64 - 20_000).abs() <= 1000);
    let time_used = simulate_blocks(5_000, 1000.into());
    assert!((time_used as i64 - 5_000).abs() <= 1000);
}

#[stest::test]
fn verify_header_test_barnard_block3_ubuntu22() {
    let header = BlockHeader::new(
        HashValue::from_hex_literal(
            "0xae1c7990f16e056bbaa7eb82ad0aec905a4ea0c559ca623f13c2a91403f81ecc",
        )
        .unwrap(),
        1616847038282,
        3,
        AccountAddress::from_hex_literal("0x94e957321e7bb2d3eb08ff6be81a6fcd").unwrap(),
        HashValue::from_hex_literal(
            "0x3da1d80128ea59c683cd1ca88f77b239fb46afa28e9f4b25753b147ca0cefaba",
        )
        .unwrap(),
        HashValue::from_hex_literal(
            "0x3df88e7a7b0ae0064fa284f71a3777c76aa83b30f16e8875a5b3ba1d94ca83b1",
        )
        .unwrap(),
        HashValue::from_hex_literal(
            "0x610596802d69223d593b5f708e5803c53f1b5958a25097ae7f8fe8cd52ce6e51",
        )
        .unwrap(),
        0,
        478.into(),
        HashValue::from_hex_literal(
            "0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97",
        )
        .unwrap(),
        ChainId::new(251),
        2894404328,
        BlockHeaderExtra::new([0u8; 4]),
    );
    G_CRYPTONIGHT
        .verify_header_difficulty(header.difficulty(), &header)
        .unwrap()
}

#[stest::test]
fn verify_header_test_barnard_block8_ubuntu22() {
    let header = BlockHeader::new(
        HashValue::from_hex_literal(
            "0xc77be02f17ae90bdbca131ef535b0eab27c2824d76968bb88a0d04a60fba9698",
        )
            .unwrap(),
        161684704549,
        8,
        AccountAddress::from_hex_literal("0x94e957321e7bb2d3eb08ff6be81a6fcd").unwrap(),
        HashValue::from_hex_literal(
            "0xf7e83302d19fef31fa15d50be2dc0d311c376f44d8dbcebf6689f9c09790d33a",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0x5f2696529b7c9c1777ec13d3c5c16425c5b60eb7de3fa85543db4127c86a4e8f",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0x8c509c07e266d643f6a6422892ace3a414ee25cf5abd3ff056ff0138a6ab818e",
        )
            .unwrap(),
        0,
        0x06f6.into(),
        HashValue::from_hex_literal(
            "0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97",
        )
            .unwrap(),
        ChainId::new(251),
        62493013,
        BlockHeaderExtra::new([0u8; 4]),
    );
    G_CRYPTONIGHT
        .verify_header_difficulty(header.difficulty(), &header)
        .unwrap()
}

#[stest::test]
fn verify_header_test_main_block2_ubuntu22() {
    let header = BlockHeader::new(
        HashValue::from_hex_literal(
            "0xc2ce2515f48a2ffef2e69b0288bb2aa78c8bfa9a55312eb948758702cd55f805",
        )
            .unwrap(),
        1621311561476,
        2,
        AccountAddress::from_hex_literal("0x000dc78e982dcdc5c80246f76d2140aa").unwrap(),
        HashValue::from_hex_literal(
            "0x40e0bb9454a40a6a2dd0579a60557c419bec995dfd80a81452a2fe0cd344ec1d",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0xe2d703b76eb066b4453eb6b05f90f9fe498d85afbafc97110cc946702bf10459",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0xa4be7b3d401da8ed15f7c4fa7c06a7cd25815f246b62fb0e7f05f32a59739c89",
        )
            .unwrap(),
        0,
        11660343.into(),
        HashValue::from_hex_literal(
            "0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97",
        )
            .unwrap(),
        ChainId::new(1),
        3221282425,
        BlockHeaderExtra::new([0u8; 4]),
    );
    G_CRYPTONIGHT
        .verify_header_difficulty(header.difficulty(), &header)
        .unwrap()
}

#[stest::test]
fn verify_header_test_main_block3_ubuntu22() {
    let header = BlockHeader::new(
        HashValue::from_hex_literal(
            "0xdda5d70eac37fa646fd9cebb274acf98fba78ff1e8afeb8b4ac11a1d90ffc84c",
        )
            .unwrap(),
        1621311660684,
        3,
        AccountAddress::from_hex_literal("0x34e2fd022578411aa8c249f8dc1da714").unwrap(),
        HashValue::from_hex_literal(
            "0xe1de0056ff0d4ff61927ed33b8a82bc4cff48c3a2eedea359d1359009cd40c45",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0x2d349fee1bb6e95279dad8879023cc09abd4e809e24cd31390a30e69b23c01ed",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0xecff23fdef37e2b4899ac78433349322518a0d08d1208bb50e5ac42ffebf3730",
        )
            .unwrap(),
        0,
        0x58f61b.into(),
        HashValue::from_hex_literal(
            "0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97",
        )
            .unwrap(),
        ChainId::new(1),
        1124080139,
        BlockHeaderExtra::new([0u8; 4]),
    );
    G_CRYPTONIGHT
        .verify_header_difficulty(header.difficulty(), &header)
        .unwrap()
}

#[stest::test]
fn verify_header_test_main_block9_ubuntu22() {
    let header = BlockHeader::new(
        HashValue::from_hex_literal(
            "0xdda5d70eac37fa646fd9cebb274acf98fba78ff1e8afeb8b4ac11a1d90ffc84c",
        )
            .unwrap(),
        1621311660684,
        3,
        AccountAddress::from_hex_literal("0x34e2fd022578411aa8c249f8dc1da714").unwrap(),
        HashValue::from_hex_literal(
            "0xe1de0056ff0d4ff61927ed33b8a82bc4cff48c3a2eedea359d1359009cd40c45",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0x2d349fee1bb6e95279dad8879023cc09abd4e809e24cd31390a30e69b23c01ed",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0xecff23fdef37e2b4899ac78433349322518a0d08d1208bb50e5ac42ffebf3730",
        )
            .unwrap(),
        0,
        0x58f61b.into(),
        HashValue::from_hex_literal(
            "0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97",
        )
            .unwrap(),
        ChainId::new(1),
        1124080139,
        BlockHeaderExtra::new([0u8; 4]),
    );
    G_CRYPTONIGHT
        .verify_header_difficulty(header.difficulty(), &header)
        .unwrap()
}
#[stest::test]
fn verify_header_test_main_block10_ubuntu22() {
    let header = BlockHeader::new(
        HashValue::from_hex_literal(
            "0x34e51fcc0435a6c9f21d13c05281d37103e0aa684ebec5d8b03c0acaa3421b57",
        )
            .unwrap(),
        1621311702957,
        10,
        AccountAddress::from_hex_literal("0x34e2fd022578411aa8c249f8dc1da714").unwrap(),
        HashValue::from_hex_literal(
            "0x81feec936ed7c20f64d7d636c1d8abeee67df79ae77fbac4a1e0e34d5bfc0222",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0xe2e31f07c7a00b4eae270f746516790e7000d8f49540b4fef3216339fef5035a",
        )
            .unwrap(),
        HashValue::from_hex_literal(
            "0x842e4affbb7b1a897fd7f8649f34a67a07bd26c1a063d942b608296f03e04163",
        )
            .unwrap(),
        0,
        0x20597d.into(),
        HashValue::from_hex_literal(
            "0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97",
        )
            .unwrap(),
        ChainId::new(1),
        1957359399,
        BlockHeaderExtra::new([0u8; 4]),
    );
    G_CRYPTONIGHT
        .verify_header_difficulty(header.difficulty(), &header)
        .unwrap()
}

fn simulate_blocks(time_plan: u64, init_difficulty: U256) -> u64 {
    fn liner_hash_pow(difficulty: U256, current: u64) -> u64 {
        let ts = MockTimeService::new_with_value(current);
        let used_time = difficulty.as_u64();
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

#[stest::test]
fn test_next_target_zero_one() {
    assert!(
        get_next_target_helper(vec![], 10_000).is_err(),
        "expect get_next_target_helper error"
    );
    let target0: U256 = 10000.into();
    let next_target = get_next_target_helper(
        vec![BlockDiffInfo {
            timestamp: 0,
            target: target0,
        }],
        10_000,
    )
    .unwrap();

    assert_eq!(next_target, target0);
}

#[stest::test]
fn test_next_target_two_no_change() {
    //time used match the plan
    let time_plan = 10_000;
    let target0: U256 = 10000.into();
    let target1: U256 = 10000.into();
    let next_target = get_next_target_helper(
        vec![
            BlockDiffInfo {
                timestamp: time_plan,
                target: target1,
            },
            BlockDiffInfo {
                timestamp: 0,
                target: target0,
            },
        ],
        time_plan,
    )
    .unwrap();
    assert_eq!(next_target, target0);
}

#[stest::test]
fn test_next_target_two_increment_difficulty() {
    let time_plan = 10_000;
    let target0: U256 = 10000.into();
    let target1: U256 = 10000.into();
    let next_target = get_next_target_helper(
        vec![
            BlockDiffInfo {
                timestamp: time_plan / 2,
                target: target1,
            },
            BlockDiffInfo {
                timestamp: 0,
                target: target0,
            },
        ],
        time_plan,
    )
    .unwrap();
    assert!(next_target < target0);
}

#[stest::test]
fn test_next_target_two_reduce_difficulty() {
    let time_plan = 10_000;
    let target0: U256 = 10000.into();
    let target1: U256 = 10000.into();
    let next_target = get_next_target_helper(
        vec![
            BlockDiffInfo {
                timestamp: time_plan * 2,
                target: target1,
            },
            BlockDiffInfo {
                timestamp: 0,
                target: target0,
            },
        ],
        time_plan,
    )
    .unwrap();
    assert!(next_target > target0);
}

#[stest::test]
fn test_next_target_many_no_change() {
    let time_plan = 10_000;
    let target0: U256 = 10000.into();
    let blocks = (0..24)
        .rev()
        .map(|i| BlockDiffInfo {
            timestamp: time_plan * i,
            target: target0,
        })
        .collect::<Vec<_>>();
    let next_target = get_next_target_helper(blocks, time_plan).unwrap();
    assert_eq!(next_target, target0);
}

#[stest::test]
fn test_next_target_many_increment_difficulty() {
    let time_plan = 10_000;
    let target0: U256 = 10000.into();
    let blocks = (0..24)
        .rev()
        .map(|i| BlockDiffInfo {
            timestamp: (time_plan / 2) * i,
            target: target0,
        })
        .collect::<Vec<_>>();
    let next_target = get_next_target_helper(blocks, time_plan).unwrap();
    assert!(next_target < target0);
}

#[stest::test]
fn test_next_target_many_reduce_difficulty() {
    let time_plan = 10_000;
    let target0: U256 = 10000.into();
    let blocks = (0..24)
        .rev()
        .map(|i| BlockDiffInfo {
            timestamp: (time_plan * 2) * i,
            target: target0,
        })
        .collect::<Vec<_>>();
    let next_target = get_next_target_helper(blocks, time_plan).unwrap();
    assert!(next_target > target0);
}

#[stest::test]
fn test_next_target_increment_difficulty_compare() {
    let time_plan = 10_000;
    let target0: U256 = 10000.into();
    let blocks_1 = vec![
        BlockDiffInfo {
            timestamp: time_plan * 2,
            target: target0,
        },
        BlockDiffInfo {
            timestamp: time_plan + 5000,
            target: target0,
        },
        BlockDiffInfo {
            timestamp: 0,
            target: target0,
        },
    ];

    let blocks_2 = vec![
        BlockDiffInfo {
            timestamp: time_plan * 2,
            target: target0,
        },
        BlockDiffInfo {
            timestamp: time_plan - 5000,
            target: target0,
        },
        BlockDiffInfo {
            timestamp: 0,
            target: target0,
        },
    ];

    let next_target_1 = get_next_target_helper(blocks_1, time_plan).unwrap();
    let next_target_2 = get_next_target_helper(blocks_2, time_plan).unwrap();
    assert!(next_target_1 < next_target_2);
    assert!(next_target_1 < target0);
    assert!(next_target_2 > target0);
}
