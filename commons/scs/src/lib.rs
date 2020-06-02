// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # Starcoin Canonical Serialization (SCS)
//!
//! SCS defines a deterministic means for translating a message or data structure into bytes
//! irrespective of platform, architecture, or programming language.

// Just a wrap to Libra Canonical Serialization (LCS) currently.
use anyhow::Result;
pub use lcs::MAX_SEQUENCE_LENGTH;
use serde::{Deserialize, Serialize};

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    lcs::to_bytes(value).map_err(|e| e.into())
}

pub fn from_bytes<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    lcs::from_bytes(bytes).map_err(|e| e.into())
}

pub trait SCSCodec<'a>: Sized {
    fn encode(&self) -> Result<Vec<u8>>;
    fn decode(bytes: &'a [u8]) -> Result<Self>;
}

impl<'a, T> SCSCodec<'a> for T
where
    T: Serialize + Deserialize<'a>,
{
    fn encode(&self) -> Result<Vec<u8>> {
        to_bytes(self)
    }

    fn decode(bytes: &'a [u8]) -> Result<Self> {
        from_bytes(bytes)
    }
}

pub use lcs::{is_human_readable, serialize_into, serialized_size};

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct Test {
        f: u64,
    }

    #[test]
    fn test_serialize() {
        let t1 = Test { f: 1 };
        let bytes = t1.encode().unwrap();
        let t2 = Test::decode(bytes.as_slice()).unwrap();
        assert_eq!(t1, t2);
    }
}
