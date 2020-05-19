// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::CORE_CODE_ADDRESS;
use crate::{
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag, TypeTag},
};
use anyhow::Result;
use once_cell::sync::Lazy;

pub const COIN_MODULE_NAME: &str = "Coin";
static COIN_MODULE_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new(COIN_MODULE_NAME).unwrap());
static COIN_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());
pub static COIN_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, COIN_MODULE_IDENTIFIER.clone()));

pub fn coin_module_name() -> &'static IdentStr {
    &*COIN_MODULE_IDENTIFIER
}

pub fn coin_struct_name() -> &'static IdentStr {
    &*COIN_STRUCT_NAME
}

// TODO: This imposes a few implied restrictions:
//   1) The currency module must be published under the core code address.
//   2) The module name must be the same as the gas specifier.
//   3) The struct name must be "T"
// We need to consider whether we want to switch to a more or fully qualified name.
pub fn type_tag_for_currency_code(currency_code: Identifier) -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: currency_code,
        name: coin_struct_name().to_owned(),
        type_params: vec![],
    })
}

pub fn from_currency_code_string(currency_code_string: &str) -> Result<Identifier> {
    Identifier::new(currency_code_string)
}
