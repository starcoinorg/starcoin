// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher, PlainCryptoHash},
    HashValue,
};

#[derive(Debug, Hash, Serialize, Deserialize, CryptoHasher, CryptoHash)]
struct TestStruct {
    str_field: String,
}

#[test]
fn test_crypto_hash() {
    let o = TestStruct {
        str_field: "hello".to_string(),
    };

    let o2 = TestStruct {
        str_field: "hello".to_string(),
    };

    let hash1 = o.crypto_hash();
    let hash2 = o2.crypto_hash();
    assert_eq!(hash1, hash2);
}

// #[test]
// fn test_crypto_hash_for_basic_type() {
//     assert_eq!("test".crypto_hash(), "test".to_string().crypto_hash());
//     assert_eq!(
//         b"test".to_vec().as_slice().crypto_hash(),
//         b"test".to_vec().crypto_hash()
//     );
// }
