use crate::consensus::Consensus;
use crate::ARGON;
use starcoin_crypto::hash::PlainCryptoHash;
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
