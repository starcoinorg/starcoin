// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod config;

pub use config::{
    GenesisConfig, G_BARNARD_CONFIG, G_DAG_TEST_CONFIG, G_DEV_CONFIG, G_HALLEY_CONFIG,
    G_MAIN_CONFIG, G_PROXIMA_CONFIG, G_TEST_CONFIG, G_VEGA_CONFIG,
};

use anyhow::{format_err, Result};
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{self, Formatter};
use std::fmt::{Debug, Display};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, Default)]
pub enum StdlibVersion {
    #[default]
    Latest,
    Version(VersionNumber),
}

type VersionNumber = u64;

impl StdlibVersion {
    pub fn new(version: u64) -> Self {
        if version == 0 {
            Self::Latest
        } else {
            Self::Version(version)
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Self::Latest => "latest".to_string(),
            Self::Version(version) => format!("{}", version),
        }
    }

    pub fn version(&self) -> u64 {
        match self {
            Self::Latest => 0,
            Self::Version(version) => *version,
        }
    }

    pub fn is_latest(&self) -> bool {
        matches!(self, Self::Latest)
    }

    pub fn compatible_with_previous(version: &Self) -> bool {
        // currently only 4 is not compatible with previous version
        // Todo: need a better solution
        !matches!(version, Self::Version(4))
    }
}

impl PartialOrd for StdlibVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StdlibVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Latest, Self::Latest) => Ordering::Equal,
            (Self::Latest, _) => Ordering::Greater,
            (_, Self::Latest) => Ordering::Less,
            (Self::Version(self_v), Self::Version(other_v)) => self_v.cmp(other_v),
        }
    }
}

impl FromStr for StdlibVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            s => Ok(Self::new(s.parse()?)),
        }
    }
}

impl Display for StdlibVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latest => f.write_str("latest"),
            Self::Version(version) => f.write_str(version.to_string().as_str()),
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
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
#[derive(Default)]
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

impl fmt::Display for ConsensusStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dummy => write!(f, "dummy"),
            Self::Argon => write!(f, "argon"),
            Self::Keccak => write!(f, "keccak"),
            Self::CryptoNight => write!(f, "cryptonight"),
        }
    }
}

impl FromStr for ConsensusStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dummy" => Ok(Self::Dummy),
            "argon" => Ok(Self::Argon),
            "keccak" => Ok(Self::Keccak),
            "cryptonight" => Ok(Self::CryptoNight),
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
        Self::new(255)
    }

    pub fn is_main(self) -> bool {
        //TODO find way share the id define with BuiltinNetworkID
        self.id == 1
    }
    pub fn is_vega(self) -> bool {
        self.id == 2
    }

    pub fn is_dag_test(self) -> bool {
        self.id == 250
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
        Ok(Self::new(id))
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

impl MoveStructType for ChainId {
    const MODULE_NAME: &'static IdentStr = ident_str!("chain_id");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ChainId");
}

impl MoveResource for ChainId {}
