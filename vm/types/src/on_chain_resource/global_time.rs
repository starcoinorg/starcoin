// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::{Deserialize, Serialize};

use schemars::JsonSchema;
const TIMESTAMP_MODULE_NAME: &str = "Timestamp";

/// The CurrentTimeMilliseconds on chain.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct GlobalTimeOnChain {
    pub milliseconds: u64,
}

impl GlobalTimeOnChain {
    pub fn new(milliseconds: u64) -> Self {
        Self { milliseconds }
    }
}

impl GlobalTimeOnChain {
    pub fn seconds(&self) -> u64 {
        self.milliseconds / 1000
    }
}

impl MoveStructType for GlobalTimeOnChain {
    const STRUCT_NAME: &'static IdentStr = ident_str!("CurrentTimeMilliseconds");
    const MODULE_NAME: &'static IdentStr = ident_str!(TIMESTAMP_MODULE_NAME);
}

impl MoveResource for GlobalTimeOnChain {}
