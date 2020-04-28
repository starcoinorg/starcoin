// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use libra_crypto::hash::HashValue;

pub use crypto_macro::CryptoHash;

pub use libra_crypto::hash::CryptoHash as LibraCryptoHash;

/// A type that implements `CryptoHash` can be hashed by a cryptographic hash function and produce
/// a `HashValue`.
pub trait CryptoHash {
    /// Hashes the object and produces a `HashValue`.
    fn crypto_hash(&self) -> HashValue;
}

impl CryptoHash for &str {
    fn crypto_hash(&self) -> HashValue {
        HashValue::from_sha3_256(
            scs::to_bytes(self)
                .expect("Serialization should work.")
                .as_slice(),
        )
    }
}

impl CryptoHash for String {
    fn crypto_hash(&self) -> HashValue {
        HashValue::from_sha3_256(
            scs::to_bytes(self)
                .expect("Serialization should work.")
                .as_slice(),
        )
    }
}

impl CryptoHash for Vec<u8> {
    fn crypto_hash(&self) -> HashValue {
        HashValue::from_sha3_256(
            scs::to_bytes(self)
                .expect("Serialization should work.")
                .as_slice(),
        )
    }
}

impl CryptoHash for &[u8] {
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
