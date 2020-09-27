// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::set_header_nonce;
use crate::ARGON;
use proptest::{collection::vec, prelude::*};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_types::block::{BlockHeader, RawBlockHeader};

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
    let nonce = ARGON.solve_consensus_nonce(raw_header.crypto_hash(), raw_header.difficulty);
    header.nonce = nonce;
    ARGON
        .verify_header_difficulty(header.difficulty, &header)
        .unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_calculate_hash(
        hashes in any::<HashValue>(),
        nonce in any::<u64>()) {
            let result = ARGON.calculate_pow_hash(hashes, nonce);
            assert!(result.is_ok());
    }

    #[test]
    fn test_set_header_nonce(
        header in vec(any::<u8>(), 0..256),
        nonce in any::<u64>()) {
            let input = set_header_nonce(header.to_vec().as_slice(), nonce);
            if header.len() > 7 {
                assert!(!input.is_empty());
            }else {
                assert!(input.is_empty());
            }
    }
}
