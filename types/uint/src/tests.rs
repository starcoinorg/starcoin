use super::*;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockHeader;

#[test]
fn test() {
    let u = U256::from(1024);
    let encoded = u.encode().unwrap();
    let u_decoded = U256::decode(&encoded).unwrap();
    assert_eq!(u, u_decoded);
    let max_hash: HashValue = U256::max_value().into();
    let max_u256: U256 = max_hash.into();
    assert_eq!(max_u256, U256::max_value());
    let h = BlockHeader::random();
    let h_encode = h.encode().unwrap();
    let h_decode = BlockHeader::decode(&h_encode).unwrap();
    assert_eq!(h, h_decode);
    let human_encode = serde_json::to_string_pretty(&U256::max_value()).unwrap();
    let human_decode: U256 = serde_json::from_str(&human_encode).unwrap();
    assert_eq!(human_decode, U256::max_value());
    assert_eq!(
        "\"0x0400\"",
        serde_json::to_string_pretty(&U256::from(1024)).unwrap()
    );
    assert_eq!(
        "\"0x00\"",
        serde_json::to_string_pretty(&U256::from(0)).unwrap()
    );
    assert_eq!(U256::from(0), serde_json::from_str("\"0x00\"").unwrap());
}
