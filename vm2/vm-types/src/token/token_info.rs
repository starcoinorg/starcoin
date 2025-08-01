// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::DataPath;
use crate::language_storage::StructTag;
use crate::language_storage::TypeTag;
use crate::{
    access_path::AccessPath, account_config::constants::CORE_CODE_ADDRESS, event::EventHandle,
};
use anyhow::Result;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::{Deserialize, Serialize};

/// Struct that represents a TokenInfo resource
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub total_value: u128,
    pub scaling_factor: u128,
    pub mint_events: EventHandle,
    pub burn_events: EventHandle,
}

impl MoveStructType for TokenInfo {
    const MODULE_NAME: &'static IdentStr = ident_str!("Token");
    const STRUCT_NAME: &'static IdentStr = ident_str!("TokenInfo");
}

impl MoveResource for TokenInfo {}

impl TokenInfo {
    pub fn total_value(&self) -> u128 {
        self.total_value
    }
    pub fn scaling_factor(&self) -> u128 {
        self.scaling_factor
    }

    pub fn struct_tag_for(token_type_tag: StructTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Self::module_identifier(),
            name: Self::struct_identifier(),
            type_args: vec![TypeTag::Struct(Box::new(token_type_tag))],
        }
    }

    pub fn resource_path_for(token_type_tag: StructTag) -> AccessPath {
        AccessPath::resource_access_path(
            token_type_tag.address,
            Self::struct_tag_for(token_type_tag),
        )
    }

    pub fn data_path_for(token_type_tag: StructTag) -> DataPath {
        AccessPath::resource_data_path(Self::struct_tag_for(token_type_tag))
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}
