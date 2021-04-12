// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Just a wrap to BCS currently.
use anyhow::Result;
pub use bcs::MAX_SEQUENCE_LENGTH;
use serde::{Deserialize, Serialize};

pub mod test_helpers {
    pub use bcs::test_helpers::*;
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    bcs::to_bytes(value).map_err(|e| e.into())
}

pub fn from_bytes<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    bcs::from_bytes(bytes).map_err(|e| e.into())
}

#[allow(clippy::upper_case_acronyms)]
pub trait BCSCodec<'a>: Sized {
    fn encode(&self) -> Result<Vec<u8>>;
    fn decode(bytes: &'a [u8]) -> Result<Self>;
}

impl<'a, T> BCSCodec<'a> for T
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

pub use bcs::{is_human_readable, serialize_into, serialized_size, Error};

pub trait Sample {
    /// A default construct for generate type Sample data for test or document.
    /// Please ensure return same data when call sample fn.
    fn sample() -> Self;
}

#[cfg(test)]
mod tests;
