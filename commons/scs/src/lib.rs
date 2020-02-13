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

pub trait SCSToBytes {
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

impl<T> SCSToBytes for T
where
    T: Serialize,
{
    fn to_bytes(&self) -> Result<Vec<u8>> {
        to_bytes(self)
    }
}

pub trait SCSFromBytes<'a>: Sized {
    fn from_bytes(bytes: &'a [u8]) -> Result<Self>;
}

impl<'a, T> SCSFromBytes<'a> for T
where
    T: Deserialize<'a>,
{
    fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        from_bytes(bytes).map_err(|e| e.into())
    }
}

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
        let bytes = t1.to_bytes().unwrap();
        let t2 = Test::from_bytes(bytes.as_slice()).unwrap();
        assert_eq!(t1, t2);
    }
}
