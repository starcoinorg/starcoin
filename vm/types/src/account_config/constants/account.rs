// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::constants::CORE_CODE_ADDRESS,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
};
use once_cell::sync::Lazy;

pub const ACCOUNT_MODULE_NAME: &str = "account";

// Account
static G_ACCOUNT_MODULE_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Account").unwrap());

static G_ACCOUNT_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Account").unwrap());

static G_ACCOUNT_BALANCE_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Balance").unwrap());

/// The ModuleId for the Account module.
pub static G_ACCOUNT_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, G_ACCOUNT_MODULE_IDENTIFIER.clone()));

pub fn account_struct_tag() -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: G_ACCOUNT_MODULE_IDENTIFIER.clone(),
        name: G_ACCOUNT_STRUCT_NAME.to_owned(),
        type_args: vec![],
    }
}
