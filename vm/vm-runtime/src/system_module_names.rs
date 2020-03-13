// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Names of modules and functions used by Libra System.

use types::{account_config, identifier::Identifier,language_storage::ModuleId};
use move_core_types::identifier::Identifier as LibraIdentifier;
use once_cell::sync::Lazy;
use libra_types::{language_storage::ModuleId as LibraModuleId};

// Data to resolve basic account and transaction flow functions and structs
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
/// The ModuleId for the LibraCoin module
pub static COIN_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("LibraCoin").unwrap(),
    );
    module_id.into()
});
/// The ModuleId for the Event
pub static EVENT_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("Event").unwrap(),
    );
    module_id.into()
});
/// The ModuleId for the validator config
pub static VALIDATOR_CONFIG_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("ValidatorConfig").unwrap(),
    );
    module_id.into()
});
/// The ModuleId for the libra system module
pub static LIBRA_SYSTEM_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("LibraSystem").unwrap(),
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
/// The ModuleId for the transaction fee module
pub static TRANSACTION_FEE_MODULE: Lazy<LibraModuleId> = Lazy::new(|| {
    let module_id = ModuleId::new(
        account_config::core_code_address(),
        Identifier::new("TransactionFee").unwrap(),
    );
    module_id.into()
});

// Names for special functions and structs
pub static CREATE_ACCOUNT_NAME: Lazy<LibraIdentifier> =
    Lazy::new(|| LibraIdentifier::new("create_account").unwrap());
pub static PROLOGUE_NAME: Lazy<LibraIdentifier> = Lazy::new(|| LibraIdentifier::new("prologue").unwrap());
pub static EPILOGUE_NAME: Lazy<LibraIdentifier> = Lazy::new(|| LibraIdentifier::new("epilogue").unwrap());
pub static BLOCK_PROLOGUE: Lazy<LibraIdentifier> =
    Lazy::new(|| LibraIdentifier::new("block_prologue").unwrap());
