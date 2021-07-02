// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::DataPath;
use crate::token::token_code::TokenCode;
use crate::{
    access_path::AccessPath,
    account_config::constants::{stc_type_tag, ACCOUNT_MODULE_NAME, CORE_CODE_ADDRESS},
    move_resource::MoveResource,
};
use move_core_types::language_storage::{StructTag, TypeTag};
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
    pub fn struct_tag_for_token(token_type_tag: StructTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: BalanceResource::struct_identifier(),
            module: BalanceResource::module_identifier(),
            type_params: vec![TypeTag::Struct(token_type_tag)],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn access_path_for(token_type_tag: StructTag) -> DataPath {
        AccessPath::resource_data_path(BalanceResource::struct_tag_for_token(token_type_tag))
    }

    /// Get token code from Balance StructTag, return None if struct tag is not a valid Balance StructTag
    pub fn token_code(struct_tag: &StructTag) -> Option<TokenCode> {
        if struct_tag.address == CORE_CODE_ADDRESS
            && struct_tag.module.as_str() == Self::MODULE_NAME
            && struct_tag.name.as_str() == Self::STRUCT_NAME
        {
            if let Some(TypeTag::Struct(token_tag)) = struct_tag.type_params.get(0) {
                Some(token_tag.clone().into())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl MoveResource for BalanceResource {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "Balance";

    fn type_params() -> Vec<TypeTag> {
        vec![stc_type_tag()]
    }
}
