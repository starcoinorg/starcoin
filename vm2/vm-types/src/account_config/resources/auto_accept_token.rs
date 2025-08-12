// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::{ACCOUNT_MODULE_NAME, CORE_CODE_ADDRESS};
use crate::token::token_code::TokenCode;
use move_core_types::ident_str;
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::{Deserialize, Serialize};

/// The AutoAcceptToken resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct AutoAcceptToken {
    enable: bool,
}

impl AutoAcceptToken {
    pub fn enable(&self) -> bool {
        self.enable
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

impl MoveStructType for AutoAcceptToken {
    const MODULE_NAME: &'static IdentStr = ident_str!(ACCOUNT_MODULE_NAME);
    const STRUCT_NAME: &'static IdentStr = ident_str!("AutoAcceptToken");
}

impl MoveResource for AutoAcceptToken {}
