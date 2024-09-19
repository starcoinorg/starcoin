// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address::AccountAddress, account_config::constants::ACCOUNT_MODULE_NAME};
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::MoveStructType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyRotationCapabilityResource {
    account_address: AccountAddress,
}

impl KeyRotationCapabilityResource {
    pub fn account_address(&self) -> &AccountAddress {
        &self.account_address
    }
}

impl MoveStructType for KeyRotationCapabilityResource {
    const STRUCT_NAME: &'static IdentStr = ident_str!("KeyRotationCapability");
    const MODULE_NAME: &'static IdentStr = ident_str!(ACCOUNT_MODULE_NAME);
}
