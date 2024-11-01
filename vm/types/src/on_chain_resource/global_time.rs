// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::{Deserialize, Serialize};

use schemars::JsonSchema;
const TIMESTAMP_MODULE_NAME: &str = "timestamp";

/// The CurrentTimeMilliseconds on chain.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
pub struct GlobalTimeOnChain {
    pub microseconds: u64,
}

impl GlobalTimeOnChain {
    pub fn new(microseconds: u64) -> Self {
        Self { microseconds }
    }
}

impl GlobalTimeOnChain {
    pub fn seconds(&self) -> u64 {
        self.microseconds / 1000000
    }
}

impl MoveStructType for GlobalTimeOnChain {
    const MODULE_NAME: &'static IdentStr = ident_str!(TIMESTAMP_MODULE_NAME);
    const STRUCT_NAME: &'static IdentStr = ident_str!("CurrentTimeMicroseconds");
}

impl MoveResource for GlobalTimeOnChain {}
