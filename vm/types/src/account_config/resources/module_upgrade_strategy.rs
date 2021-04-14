// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use crate::account_address::AccountAddress;
use crate::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

pub const _STRATEGY_ARBITRARY: u8 = 0;
pub const STRATEGY_TWO_PHASE: u8 = 1;
pub const STRATEGY_NEW_MODULE: u8 = 2;
pub const STRATEGY_ENFORCED: u8 = 3;
pub const _STRATEGY_FREEZE: u8 = 4;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleUpgradeStrategy {
    strategy: u8,
}

impl ModuleUpgradeStrategy {
    pub fn only_new_module(&self) -> bool {
        self.strategy == STRATEGY_NEW_MODULE
    }

    pub fn two_phase(&self) -> bool {
        self.strategy == STRATEGY_TWO_PHASE
    }

    pub fn enforced(&self) -> bool {
        self.strategy == STRATEGY_ENFORCED
    }
}

impl MoveResource for ModuleUpgradeStrategy {
    const MODULE_NAME: &'static str = "PackageTxnManager";
    const STRUCT_NAME: &'static str = "ModuleUpgradeStrategy";
}

pub fn access_path_for_module_upgrade_strategy(address: AccountAddress) -> AccessPath {
    AccessPath::resource_access_path(address, ModuleUpgradeStrategy::struct_tag())
}
