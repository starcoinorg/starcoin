// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::{coin_struct_name, constants::CORE_CODE_ADDRESS, from_currency_code_string},
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use once_cell::sync::Lazy;

pub const STC_NAME: &str = "STC";
pub static STC_IDENTIFIER: Lazy<Identifier> = Lazy::new(|| Identifier::new(STC_NAME).unwrap());
pub static STC_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, STC_IDENTIFIER.to_owned()));
pub static STC_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

pub fn stc_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: from_currency_code_string(STC_NAME).unwrap(),
        name: coin_struct_name().to_owned(),
        type_params: vec![],
    })
}
