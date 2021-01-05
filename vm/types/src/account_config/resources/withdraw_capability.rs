// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_resource::MoveResource;
use crate::{account_address::AccountAddress, account_config::constants::ACCOUNT_MODULE_NAME};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawCapabilityResource {
    account_address: AccountAddress,
}

impl WithdrawCapabilityResource {
    pub fn account_address(&self) -> &AccountAddress {
        &self.account_address
    }
}

impl MoveResource for WithdrawCapabilityResource {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "WithdrawCapability";
}
