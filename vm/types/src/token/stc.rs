// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::token::token_code::TokenCode;
use crate::{
    account_config::constants::CORE_CODE_ADDRESS,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use once_cell::sync::Lazy;
use std::str::FromStr;

pub const STC_NAME: &str = "STC";
pub const STC_TOKEN_CODE_STR: &str = "0x1::STC::STC";
pub const WRONG_TOKEN_CODE_STR_FOR_TEST: &str = "0x1::ABC::ABC";

pub static STC_TOKEN_CODE: Lazy<TokenCode> = Lazy::new(|| {
    TokenCode::from_str(STC_TOKEN_CODE_STR).expect("Parse STC token code should success.")
});

pub static WRONG_TOKEN_CODE_FOR_TEST: Lazy<TokenCode> = Lazy::new(|| {
    TokenCode::from_str(WRONG_TOKEN_CODE_STR_FOR_TEST)
        .expect("Parse wrong token code should success.")
});

static STC_IDENTIFIER: Lazy<Identifier> = Lazy::new(|| Identifier::new(STC_NAME).unwrap());

pub fn stc_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: STC_IDENTIFIER.clone(),
        name: STC_IDENTIFIER.clone(),
        type_params: vec![],
    })
}
