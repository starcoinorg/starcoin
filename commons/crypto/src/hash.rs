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

pub fn create_literal_hash(word: &str) -> HashValue {
    let mut s = word.as_bytes().to_vec();
    assert!(s.len() <= HashValue::LENGTH);
    s.resize(HashValue::LENGTH, 0);
    HashValue::from_slice(&s).expect("Cannot fail")
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

    #[test]
    fn test_crypto_hash_for_basic_type() {
        assert_eq!("test".crypto_hash(), "test".to_string().crypto_hash());
        //lcs serde::Serialize &[u8] is different with Vec<u8>, TODO conform.
        assert_ne!(b"test".crypto_hash(), b"test".to_vec().crypto_hash());
    }
}
