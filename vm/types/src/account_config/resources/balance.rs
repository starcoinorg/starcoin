// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::token::token_code::TokenCode;
use crate::{
    access_path::AccessPath,
    account_config::constants::{stc_type_tag, ACCOUNT_MODULE_NAME, CORE_CODE_ADDRESS},
};
use move_core_types::{
    language_storage::{StructTag, TypeTag},
    move_resource::MoveResource,
};
use serde::{Deserialize, Serialize};

/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResource {
    token: u128,
}

impl BalanceResource {
    pub fn new(token: u128) -> Self {
        Self { token }
    }

    pub fn token(&self) -> u128 {
        self.token
    }

    // TODO/XXX: remove this once the MoveResource trait allows type arguments to `struct_tag`.
    pub fn struct_tag_for_token(token_type_tag: TypeTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: BalanceResource::struct_identifier(),
            module: BalanceResource::module_identifier(),
            type_params: vec![token_type_tag],
        }
    }

    pub fn struct_tag_for_token_code(token_code: TokenCode) -> StructTag {
        Self::struct_tag_for_token(token_code.into())
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn access_path_for(token_type_tag: TypeTag) -> Vec<u8> {
        AccessPath::resource_access_vec(&BalanceResource::struct_tag_for_token(token_type_tag))
    }
}

impl MoveResource for BalanceResource {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "Balance";

    fn type_params() -> Vec<TypeTag> {
        vec![stc_type_tag()]
    }
}
