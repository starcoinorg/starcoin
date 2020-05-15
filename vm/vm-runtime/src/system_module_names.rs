// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Names of modules and functions used by Libra System.

use once_cell::sync::Lazy;
use starcoin_types::account_config;
use starcoin_vm_types::{identifier::Identifier, language_storage::ModuleId};

// Data to resolve basic account and transaction flow functions and structs

/// LBR
static LBR_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("LBR").unwrap());
pub static LBR_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(account_config::core_code_address(), LBR_MODULE_NAME.clone()));

/// Starcoin
static STARCOIN_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Starcoin").unwrap());
pub static STARCOIN_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        STARCOIN_MODULE_NAME.clone(),
    )
});

pub static ACCOUNT_MODULE_NAME: &str = "LibraAccount";

/// The ModuleId for the Account module
pub static ACCOUNT_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new(ACCOUNT_MODULE_NAME).unwrap(),
    )
});
/// The ModuleId for the LibraTransactionTimeout module
pub static LIBRA_TRANSACTION_TIMEOUT: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("LibraTransactionTimeout").unwrap(),
    )
});

pub static SUBSIDY_CONF_MODULE_NAME: &str = "SubsidyConfig";

/// The ModuleId for the subsidy config module
pub static SUBSIDY_CONF_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new(SUBSIDY_CONF_MODULE_NAME).unwrap(),
    )
});

pub static LIBRA_BLOCK_MODULE_NAME: &str = "LibraBlock";

/// The ModuleId for the libra block module
pub static LIBRA_BLOCK_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new(LIBRA_BLOCK_MODULE_NAME).unwrap(),
    )
});
/// The ModuleId for the gas schedule module
pub static GAS_SCHEDULE_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("GasSchedule").unwrap(),
    )
});

pub static GAS_SCHEDULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

// Names for special functions and structs
pub static CREATE_ACCOUNT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("create_account").unwrap());
pub static PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static EPILOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static BLOCK_PROLOGUE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
