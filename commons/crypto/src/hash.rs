// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use libra_crypto::hash::HashValue;

/// A type that implements `CryptoHash` can be hashed by a cryptographic hash function and produce
/// a `HashValue`.
pub trait CryptoHash {
    /// Hashes the object and produces a `HashValue`.
    fn crypto_hash(&self) -> HashValue;
}

impl<T: serde::Serialize> CryptoHash for T {
    fn crypto_hash(&self) -> HashValue {
        HashValue::from_sha3_256(
            scs::to_bytes(self)
                .expect("Serialization should work.")
                .as_slice(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Hash, Serialize, Deserialize)]
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
}
