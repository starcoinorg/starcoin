// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use crate::account_address::AccountAddress;
use crate::event::EventHandle;
use crate::move_resource::MoveResource;
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
    AccessPath::resource_access_path(address, ModuleUpgradeStrategy::struct_tag())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoPhaseUpgradeV2Resource {
    config: TwoPhaseUpgradeConfigResource,
    plan: Option<UpgradePlanV2Resource>,
    version_cap: ModifyConfigCapabilityResource,
    upgrade_event: EventHandle,
}
impl TwoPhaseUpgradeV2Resource {
    pub fn enforced(&self) -> bool {
        match &self.plan {
            Some(plan) => plan.enforced,
            None => false,
        }
    }
}
impl MoveResource for TwoPhaseUpgradeV2Resource {
    const MODULE_NAME: &'static str = "PackageTxnManager";
    const STRUCT_NAME: &'static str = "TwoPhaseUpgradeV2";
}

pub fn access_path_for_two_phase_upgrade_v2(address: AccountAddress) -> AccessPath {
    AccessPath::resource_access_path(address, TwoPhaseUpgradeV2Resource::struct_tag())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoPhaseUpgradeConfigResource {
    min_time_limit: u64,
}
impl MoveResource for TwoPhaseUpgradeConfigResource {
    const MODULE_NAME: &'static str = "PackageTxnManager";
    const STRUCT_NAME: &'static str = "TwoPhaseUpgradeConfig";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradePlanV2Resource {
    package_hash: Vec<u8>,
    active_after_time: u64,
    version: u64,
    enforced: bool,
}
impl MoveResource for UpgradePlanV2Resource {
    const MODULE_NAME: &'static str = "PackageTxnManager";
    const STRUCT_NAME: &'static str = "UpgradePlanV2";
}

#[derive(Debug, Serialize, Deserialize)]
struct ModifyConfigCapabilityResource {
    account_address: AccountAddress,
    events: EventHandle,
}
impl MoveResource for ModifyConfigCapabilityResource {
    const MODULE_NAME: &'static str = "PackageTxnManager";
    const STRUCT_NAME: &'static str = "ModifyConfigCapability";
}
