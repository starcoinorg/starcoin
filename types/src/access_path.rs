// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use starcoin_vm2_vm_types::access_path::AccessPath as AccessPath2;
pub use starcoin_vm_types::access_path::{AccessPath, DataPath, DataType};

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, JsonSchema)]
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
