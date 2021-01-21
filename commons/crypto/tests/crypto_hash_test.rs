// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher, PlainCryptoHash};

#[derive(Debug, Hash, Serialize, Deserialize, CryptoHasher, CryptoHash)]
struct TestStruct {
    str_field: String,
}

#[derive(Debug, Hash, Serialize, Deserialize, CryptoHasher, CryptoHash)]
struct TestStruct2 {
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

    let o3 = TestStruct2 {
        str_field: "hello".to_string(),
    };

    assert_eq!(
        bcs_ext::to_bytes(&o).unwrap(),
        bcs_ext::to_bytes(&o3).unwrap()
    );
    let hash3 = o3.crypto_hash();
    assert_ne!(hash1, hash3);
}
