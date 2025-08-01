// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
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

impl MoveStructType for NewBlockEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!("Block");
    const STRUCT_NAME: &'static IdentStr = ident_str!("NewBlockEvent");
}

impl MoveResource for NewBlockEvent {}
