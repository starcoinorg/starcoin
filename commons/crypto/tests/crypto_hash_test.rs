// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};

#[derive(Debug, Hash, Serialize, Deserialize, CryptoHash)]
struct TestStruct {
    str_field: String,
}

#[test]
fn test_crypto_hash() {
    let o = TestStruct {
        str_field: "hello".to_string(),
    };
    let hash1 = HashValue::from_sha3_256(
        scs::to_bytes(&o)
            .expect("Serialization should work.")
            .as_slice(),
    );
    let hash2 = o.crypto_hash();
    assert_eq!(hash1, hash2);
}

#[test]
fn test_crypto_hash_for_basic_type() {
    assert_eq!("test".crypto_hash(), "test".to_string().crypto_hash());
    assert_eq!(
        b"test".to_vec().as_slice().crypto_hash(),
        b"test".to_vec().crypto_hash()
    );
}
