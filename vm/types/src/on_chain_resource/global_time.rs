// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use move_core_types::move_resource::MoveResource;

const TIMESTAMP_MODULE_NAME: &str = "Timestamp";

/// The CurrentTimeMilliseconds on chain.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

impl MoveResource for GlobalTimeOnChain {
    const MODULE_NAME: &'static str = TIMESTAMP_MODULE_NAME;
    const STRUCT_NAME: &'static str = "CurrentTimeMilliseconds";
}
