// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use starcoin_vm2_vm_types::access_path::AccessPath as AccessPath2;
pub use starcoin_vm_types::access_path::{AccessPath, DataPath, DataType};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, JsonSchema)]
#[schemars(with = "String")]
pub enum StcAccessPath {
    V1(AccessPath),
    V2(AccessPath2),
}

impl From<AccessPath> for StcAccessPath {
    fn from(access_path: AccessPath) -> Self {
        StcAccessPath::V1(access_path)
    }
}

impl From<AccessPath2> for StcAccessPath {
    fn from(access_path: AccessPath2) -> Self {
        StcAccessPath::V2(access_path)
    }
}

impl TryFrom<StcAccessPath> for AccessPath {
    type Error = anyhow::Error;
    fn try_from(value: StcAccessPath) -> Result<Self, Self::Error> {
        match value {
            StcAccessPath::V1(path) => Ok(path),
            StcAccessPath::V2(_path) => anyhow::bail!("V2 AccessPath cannot be convert to V1"),
        }
    }
}

impl TryFrom<StcAccessPath> for AccessPath2 {
    type Error = anyhow::Error;
    fn try_from(value: StcAccessPath) -> Result<Self, Self::Error> {
        match value {
            StcAccessPath::V1(_path) => anyhow::bail!("V1 AccessPath cannot be convert to V2"),
            StcAccessPath::V2(path) => Ok(path),
        }
    }
}
