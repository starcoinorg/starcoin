// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

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

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
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
