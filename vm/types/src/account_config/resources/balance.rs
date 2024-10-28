// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::DataPath;
use crate::token::token_code::TokenCode;
use crate::{
    access_path::AccessPath,
    account_config::constants::{stc_type_tag, ACCOUNT_MODULE_NAME, CORE_CODE_ADDRESS},
};
use move_core_types::ident_str;
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::move_resource::{MoveResource, MoveStructType};
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
            name: Self::struct_identifier(),
            module: Self::module_identifier(),
            type_args: vec![TypeTag::Struct(Box::new(token_type_tag))],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn access_path_for(token_type_tag: StructTag) -> DataPath {
        AccessPath::resource_data_path(Self::struct_tag_for_token(token_type_tag))
    }

    /// Get token code from Balance StructTag, return None if struct tag is not a valid Balance StructTag
    pub fn token_code(struct_tag: &StructTag) -> Option<TokenCode> {
        if struct_tag.address == CORE_CODE_ADDRESS
            && struct_tag.module == Identifier::from(Self::MODULE_NAME)
            && struct_tag.name == Identifier::from(Self::STRUCT_NAME)
        {
            if let Some(TypeTag::Struct(token_tag)) = struct_tag.type_args.first() {
                Some((*(token_tag.clone())).into())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl MoveStructType for BalanceResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("coin");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinStore");
    fn type_args() -> Vec<TypeTag> {
        vec![stc_type_tag()]
    }
}

impl MoveResource for BalanceResource {}
