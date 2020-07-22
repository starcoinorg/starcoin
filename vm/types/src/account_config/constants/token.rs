// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::CORE_CODE_ADDRESS;
use crate::{
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag, TypeTag},
};
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use once_cell::sync::Lazy;

pub const TOKEN_MODULE_NAME: &str = "Token";
static TOKEN_MODULE_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new(TOKEN_MODULE_NAME).unwrap());
static TOKEN_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Token").unwrap());
pub static COIN_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, TOKEN_MODULE_IDENTIFIER.clone()));

pub fn token_module_name() -> &'static IdentStr {
    &*TOKEN_MODULE_IDENTIFIER
}

pub fn token_struct_name() -> &'static IdentStr {
    &*TOKEN_STRUCT_NAME
}

// TODO: This imposes a few implied restrictions:
//   1) The struct name must be same as module name and same as currency_code.
// We need to consider whether we want to switch to a more or fully qualified name.
pub fn type_tag_for_currency_code(
    module_address: Option<AccountAddress>,
    currency_code: Identifier,
) -> TypeTag {
    TypeTag::Struct(StructTag {
        address: module_address.unwrap_or(CORE_CODE_ADDRESS),
        module: currency_code.clone(),
        name: currency_code,
        type_params: vec![],
    })
}

pub fn from_currency_code_string(currency_code_string: &str) -> Result<Identifier> {
    Identifier::new(currency_code_string)
}
