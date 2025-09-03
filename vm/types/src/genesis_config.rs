// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_resource::MoveResource;
use anyhow::{format_err, Result};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};
use std::fmt::{Debug, Display};
use std::str::FromStr;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    IntoPrimitive,
    TryFromPrimitive,
    JsonSchema,
)]
#[repr(u8)]
#[serde(tag = "type")]
pub enum ConsensusStrategy {
    #[default]
    Dummy = 0,
    Argon = 1,
    Keccak = 2,
    CryptoNight = 3,
}

impl ConsensusStrategy {
    pub fn value(self) -> u8 {
        self.into()
    }
}

impl Display for ConsensusStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConsensusStrategy::Dummy => write!(f, "dummy"),
            ConsensusStrategy::Argon => write!(f, "argon"),
            ConsensusStrategy::Keccak => write!(f, "keccak"),
            ConsensusStrategy::CryptoNight => write!(f, "cryptonight"),
        }
    }
}

impl FromStr for ConsensusStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dummy" => Ok(ConsensusStrategy::Dummy),
            "argon" => Ok(ConsensusStrategy::Argon),
            "keccak" => Ok(ConsensusStrategy::Keccak),
            "cryptonight" => Ok(ConsensusStrategy::CryptoNight),
            s => Err(format_err!("Unknown ConsensusStrategy: {}", s)),
        }
    }
}

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, Hash, Eq, PartialEq, PartialOrd, Ord, JsonSchema,
)]
pub struct ChainId {
    id: u8,
}

impl ChainId {
    pub fn new(id: u8) -> Self {
        Self { id }
    }

    pub fn id(self) -> u8 {
        self.id
    }

    pub fn test() -> Self {
        ChainId::new(255)
    }

    pub fn is_main(self) -> bool {
        //TODO find way share the id define with BuiltinNetworkID
        self.id == 1
    }

    pub fn is_barnard(self) -> bool {
        self.id == 251
    }

    pub fn is_proxima(self) -> bool {
        self.id == 252
    }

    pub fn is_halley(self) -> bool {
        self.id == 253
    }

    pub fn is_dev(self) -> bool {
        self.id == 254
    }

    pub fn is_test(self) -> bool {
        self.id == 255
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl FromStr for ChainId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u8 = s.parse()?;
        Ok(ChainId::new(id))
    }
}

impl From<u8> for ChainId {
    fn from(id: u8) -> Self {
        Self::new(id)
    }
}

#[allow(clippy::from_over_into)]
impl Into<u8> for ChainId {
    fn into(self) -> u8 {
        self.id
    }
}

impl MoveResource for ChainId {
    const MODULE_NAME: &'static str = "ChainId";
    const STRUCT_NAME: &'static str = "ChainId";
}

#[cfg(test)]
mod tests {
    use crate::genesis_config::ConsensusStrategy;

    #[test]
    fn test_consensus_strategy() {
        assert_eq!(ConsensusStrategy::Dummy.value(), 0);
        assert_eq!(ConsensusStrategy::Argon.value(), 1);
        assert_eq!(ConsensusStrategy::Keccak.value(), 2);
        assert_eq!(ConsensusStrategy::CryptoNight.value(), 3);
    }
}
