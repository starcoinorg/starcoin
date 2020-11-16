// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use crate::account_address::AccountAddress;
use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

pub const _STRATEGY_ARBITRARY: u8 = 0;
pub const STRATEGY_TWO_PHASE: u8 = 1;
pub const STRATEGY_NEW_MODULE: u8 = 2;
pub const _STRATEGY_FREEZE: u8 = 3;

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
}

impl MoveResource for ModuleUpgradeStrategy {
    const MODULE_NAME: &'static str = "PackageTxnManager";
    const STRUCT_NAME: &'static str = "ModuleUpgradeStrategy";
}

pub fn access_path_for_module_upgrade_strategy(address: AccountAddress) -> AccessPath {
    AccessPath::new(address, ModuleUpgradeStrategy::resource_path())
}
