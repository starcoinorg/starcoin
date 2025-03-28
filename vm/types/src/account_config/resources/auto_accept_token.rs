// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::token::token_code::TokenCode;
use crate::{
    account_config::constants::{ACCOUNT_MODULE_NAME, CORE_CODE_ADDRESS},
    move_resource::MoveResource,
};
use move_core_types::language_storage::{StructTag, TypeTag};
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
            && struct_tag.module.as_str() == Self::MODULE_NAME
            && struct_tag.name.as_str() == Self::STRUCT_NAME
        {
            if let Some(TypeTag::Struct(token_tag)) = struct_tag.type_params.first() {
                Some((*(token_tag.clone())).into())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl MoveResource for AutoAcceptToken {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "AutoAcceptToken";
}
