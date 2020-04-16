// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Names of modules and functions used by Libra System.

use libra_types::language_storage::ModuleId as LibraModuleId;
use move_core_types::identifier::Identifier;
use once_cell::sync::Lazy;
use types::{account_config, language_storage::ModuleId};

// Data to resolve basic account and transaction flow functions and structs

/// LBR
static LBR_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("LBR").unwrap());
pub static LBR_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    ModuleId::new(account_config::core_code_address(), LBR_MODULE_NAME.clone()).into()
});

/// Starcoin
static STARCOIN_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Starcoin").unwrap());
pub static STARCOIN_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    ModuleId::new(account_config::core_code_address(), STARCOIN_MODULE_NAME.clone()).into()
});

/// The ModuleId for the Account module
pub static ACCOUNT_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("LibraAccount").unwrap(),
    );
    module_id.into()
});
/// The ModuleId for the LibraTransactionTimeout module
pub static LIBRA_TRANSACTION_TIMEOUT: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("LibraTransactionTimeout").unwrap(),
    );
    module_id.into()
});

/// The ModuleId for the subsidy config module
pub static SUBSIDY_CONF_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::mint_address(),
        Identifier::new("SubsidyConfig").unwrap(),
    );
    module_id.into()
});

/// The ModuleId for the libra block module
pub static LIBRA_BLOCK_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("LibraBlock").unwrap(),
    );
    module_id.into()
});
/// The ModuleId for the gas schedule module
pub static GAS_SCHEDULE_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("GasSchedule").unwrap(),
    );
    module_id.into()
});

// Names for special functions and structs
pub static CREATE_ACCOUNT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("create_account").unwrap());
pub static PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static EPILOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static BLOCK_PROLOGUE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
