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

pub static REWARD_CONF_MODULE_NAME: &str = "RewardConfig";

/// The ModuleId for the reward config module
pub static REWARD_CONF_MODULE: Lazy<ModuleId> = Lazy::new(|| {
    ModuleId::new(
        CORE_CODE_ADDRESS,
        Identifier::new(REWARD_CONF_MODULE_NAME).unwrap(),
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

// Names for special functions and structs
pub static CREATE_ACCOUNT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("create_account").unwrap());
pub static PROLOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("prologue").unwrap());
pub static EPILOGUE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
pub static BLOCK_PROLOGUE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("block_prologue").unwrap());
