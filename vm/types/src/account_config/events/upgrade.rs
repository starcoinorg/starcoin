// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::move_resource::MoveResource;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct UpgradeEvent {
    package_address: AccountAddress,
    package_hash: HashValue,
    version: u64,
}
impl UpgradeEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for UpgradeEvent {
    const MODULE_NAME: &'static str = "PackageTxnManager";
    const STRUCT_NAME: &'static str = "UpgradeEvent";
}
