// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::integer_arithmetic)]

use crate::consensus::Consensus;
use crate::difficulty::{get_next_target_helper, BlockDiffInfo};
use crate::{difficult_to_target, target_to_difficulty, CRYPTONIGHT};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_types::block::{BlockHeader, BlockHeaderBuilder, RawBlockHeader};
use starcoin_types::U256;
use starcoin_vm_types::time::{
    duration_since_epoch, MockTimeService, TimeService, TimeServiceType,
};
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
    let nonce = CRYPTONIGHT.solve_consensus_nonce(
        &header.as_pow_header_blob(),
        raw_header.difficulty,
        time_service.as_ref(),
    );
    let header = header.as_builder().with_nonce(nonce).build();
    CRYPTONIGHT
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

#[stest::test(timeout = 100)]
fn test_next_target_target() {
    let blocks_1 = vec![
        // BlockDiffInfo {
        //     timestamp: 1637911931937,
        //     target: difficult_to_target("0x128dd7af".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911903653,
        //     target: difficult_to_target("0x14a42733".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911889623,
        //     target: difficult_to_target("0x133b86e6".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911888900,
        //     target: difficult_to_target("0x11f54845".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911888227,
        //     target: difficult_to_target("0x1310dccb".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911877864,
        //     target: difficult_to_target("0x1222c9f0".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911875570,
        //     target: difficult_to_target("0x12b92b58".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911867620,
        //     target: difficult_to_target("0x124da70c".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911863497,
        //     target: difficult_to_target("0x11408ac0".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911862363,
        //     target: difficult_to_target("0x15d7bcb2".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911839267,
        //     target: difficult_to_target("0x16317044".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911832464,
        //     target: difficult_to_target("0x168d12d7".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911825582,
        //     target: difficult_to_target("0x1ac09aff".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911811002,
        //     target: difficult_to_target("0x18a2cdbd".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911809762,
        //     target: difficult_to_target("0x184f681a".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911804871,
        //     target: difficult_to_target("0x18115b5e".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911800093,
        //     target: difficult_to_target("0x1afaf67a".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911789309,
        //     target: difficult_to_target("0x19f7fe19".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911785915,
        //     target: difficult_to_target("0x180d92a4".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911784248,
        //     target: difficult_to_target("0x161206f7".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911782976,
        //     target: difficult_to_target("0x15688141".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911779023,
        //     target: difficult_to_target("0x13ec5a26".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911777159,
        //     target: difficult_to_target("0x1431ab06".into()),
        // },
        // BlockDiffInfo {
        //     timestamp: 1637911770816,
        //     target: difficult_to_target("0x135b2bde".into()),
        // },
        BlockDiffInfo {
            timestamp: 1637915608880,
            target: difficult_to_target("0x109b3b25".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915598323,
            target: difficult_to_target("0x107bac2f".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915593635,
            target: difficult_to_target("0x0fc6aaf6".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915590037,
            target: difficult_to_target("0x0e4573d9".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915589658,
            target: difficult_to_target("0x0d9c454d".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915587412,
            target: difficult_to_target("0x0d02072c".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915583955,
            target: difficult_to_target("0x0c1a3c67".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915582180,
            target: difficult_to_target("0x0b9f8d6c".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915578529,
            target: difficult_to_target("0x0bb8e472".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915571549,
            target: difficult_to_target("0x0b1d156a".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915569450,
            target: difficult_to_target("0x0ad6bdde".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915565236,
            target: difficult_to_target("0x0aa1d359".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915560700,
            target: difficult_to_target("0x0a8a9c6b".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915555504,
            target: difficult_to_target("0x0a55d2b8".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915551514,
            target: difficult_to_target("0x09ff1eb6".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915548906,
            target: difficult_to_target("0x0a9b7089".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915537703,
            target: difficult_to_target("0x0afc8034".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915528782,
            target: difficult_to_target("0x0b9f2116".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915518066,
            target: difficult_to_target("0x0c551ecb".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915507321,
            target: difficult_to_target("0x0bac0c9b".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915506330,
            target: difficult_to_target("0x0b56064f".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915503277,
            target: difficult_to_target("0x0ad8c0ab".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915501439,
            target: difficult_to_target("0x0a6fc4c9".into()),
        },
        BlockDiffInfo {
            timestamp: 1637915499540,
            target: difficult_to_target("0x0ab72a30".into()),
        },
    ];

    let next_target_1 = get_next_target_helper(blocks_1, 5918).unwrap();
    // let difficulty = U256::from("0x0e74b15e");
    let difficulty = U256::from("0f1ccd00");
    let ttt: U256 = "0f1ccd00".into();

    println!("{}", target_to_difficulty(next_target_1));
    println!("{}", difficult_to_target(difficulty));
    assert_eq!(target_to_difficulty(next_target_1), difficulty);
}
