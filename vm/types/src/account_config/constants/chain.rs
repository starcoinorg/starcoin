// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::constants::CORE_CODE_ADDRESS, identifier::Identifier,
    language_storage::ModuleId,
};
use once_cell::sync::Lazy;

/// The ModuleId for the TransactionTimeout module
pub static TRANSACTION_TIMEOUT_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        CORE_CODE_ADDRESS,
        Identifier::new("TransactionTimeout").unwrap(),
    )
});

pub static SUBSIDY_CONF_MODULE_NAME: &str = "SubsidyConfig";

/// The ModuleId for the subsidy config module
pub static SUBSIDY_CONF_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        CORE_CODE_ADDRESS,
        Identifier::new(SUBSIDY_CONF_MODULE_NAME).unwrap(),
    )
});

pub static BLOCK_MODULE_NAME: &str = "Block";

/// The ModuleId for the libra block module
pub static BLOCK_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        CORE_CODE_ADDRESS,
        Identifier::new(BLOCK_MODULE_NAME).unwrap(),
    )
});
/// The ModuleId for the gas schedule module
pub static GAS_SCHEDULE_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, Identifier::new("GasSchedule").unwrap()));

pub static GAS_SCHEDULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

// Names for special functions and structs
pub static CREATE_ACCOUNT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("create_account").unwrap());
pub static PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static EPILOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static BLOCK_PROLOGUE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
