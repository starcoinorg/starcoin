// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct NewBlockEvent {
    number: u64,
    author: AccountAddress,
    timestamp: u64,
    uncles: u64,
}
impl NewBlockEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for NewBlockEvent {
    const MODULE_NAME: &'static str = "Block";
    const STRUCT_NAME: &'static str = "NewBlockEvent";
}
