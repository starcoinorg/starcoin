use types::block::BlockHeader;

#[stest::test]
fn raw_hash_test() {
    let mut header = BlockHeader::random();
    let id_1 = header.id();
    let raw_id_1 = header.raw_hash();
    header.consensus_header = header.parent_hash.to_vec();
    let id_2 = header.id();
    let raw_id_2 = header.raw_hash();
    assert_ne!(id_1, id_2);
    assert_eq!(raw_id_1, raw_id_2);
}
