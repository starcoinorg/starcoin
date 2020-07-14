use crypto::hash::PlainCryptoHash;
use starcoin_consensus::argon::{verify, ArgonConsensus};
use traits::Consensus;
use types::block::{BlockHeader, RawBlockHeader};

#[stest::test]
fn raw_hash_test() {
    let mut header = BlockHeader::random();
    let header_1 = header.clone();
    let id_1 = header_1.id();
    let raw_1: RawBlockHeader = header_1.into();
    let raw_id_1 = raw_1.crypto_hash();
    header.consensus_header = b"aaa".to_vec();
    let header_2 = header;
    let id_2 = header_2.id();
    let raw_2: RawBlockHeader = header_2.into();
    let raw_id_2 = raw_2.crypto_hash();
    assert_ne!(id_1, id_2);
    assert_eq!(raw_id_1, raw_id_2);
}

#[stest::test]
fn verify_header_test() {
    let header = BlockHeader::random();
    let raw_header: RawBlockHeader = header.into();
    let nonce = ArgonConsensus::solve_consensus_header(
        raw_header.crypto_hash().to_vec().as_slice(),
        1.into(),
    );
    assert!(verify(
        raw_header.crypto_hash().to_vec().as_slice(),
        nonce.nonce,
        1.into(),
    ));
}
