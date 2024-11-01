// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::constants::CORE_CODE_ADDRESS, identifier::Identifier,
    language_storage::ModuleId,
};
use once_cell::sync::Lazy;

/// The ModuleId for the TransactionTimeout module
pub static G_TRANSACTION_TIMEOUT_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        CORE_CODE_ADDRESS,
        Identifier::new("transaction_timeout").unwrap(),
    )
});

pub static G_BLOCK_MODULE_NAME: &str = "stc_block";
pub static G_TRANSACTION_MANAGER_MODULE_NAME: &str = "stc_transaction_validation";

/// The ModuleId for block module
pub static G_BLOCK_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        CORE_CODE_ADDRESS,
        Identifier::new(G_BLOCK_MODULE_NAME).unwrap(),
    )
});

/// The ModuleId for transaction manager module
pub static G_TRANSACTION_VALIDATION_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        CORE_CODE_ADDRESS,
        Identifier::new(G_TRANSACTION_MANAGER_MODULE_NAME).unwrap(),
    )
});

pub static G_PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static G_EPILOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
// pub static G_EPILOGUE_V2_NAME: Lazy<Identifier> =
//     Lazy::new(|| Identifier::new("epilogue_v2").unwrap());

pub static G_BLOCK_PROLOGUE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
// pub static G_BLOCK_PROLOGUE_V2_NAME: Lazy<Identifier> =
//     Lazy::new(|| Identifier::new("block_prologue_v2").unwrap());
