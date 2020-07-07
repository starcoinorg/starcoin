use crypto::hash::PlainCryptoHash;
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
