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

pub static BLOCK_MODULE_NAME: &str = "Block";
pub static TRANSACTION_MANAGER_MODULE_NAME: &str = "TransactionManager";

/// The ModuleId for block module
pub static BLOCK_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        CORE_CODE_ADDRESS,
        Identifier::new(BLOCK_MODULE_NAME).unwrap(),
    )
});

/// The ModuleId for transaction manager module
pub static TRANSACTION_MANAGER_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        CORE_CODE_ADDRESS,
        Identifier::new(TRANSACTION_MANAGER_MODULE_NAME).unwrap(),
    )
});

pub static PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static BLOCK_PROLOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
pub static EPILOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static EPILOGUE_V2_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("epilogue_v2").unwrap());
