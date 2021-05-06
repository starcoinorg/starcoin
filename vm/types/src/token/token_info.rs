// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::DataPath;
use crate::token::token_code::TokenCode;
use crate::{
    access_path::AccessPath, account_config::constants::CORE_CODE_ADDRESS, event::EventHandle,
};
use crate::{language_storage::StructTag, move_resource::MoveResource};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Struct that represents a TokenInfo resource
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub total_value: u128,
    pub scaling_factor: u128,
    pub mint_events: EventHandle,
    pub burn_events: EventHandle,
}

impl MoveResource for TokenInfo {
    const MODULE_NAME: &'static str = "Token";
    const STRUCT_NAME: &'static str = "TokenInfo";
}

impl TokenInfo {
    pub fn total_value(&self) -> u128 {
        self.total_value
    }
    pub fn scaling_factor(&self) -> u128 {
        self.scaling_factor
    }

    pub fn struct_tag_for(token_code: TokenCode) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: TokenInfo::module_identifier(),
            name: TokenInfo::struct_identifier(),
            type_params: vec![token_code.into()],
        }
    }

    pub fn resource_path_for(token_code: TokenCode) -> AccessPath {
        AccessPath::resource_access_path(token_code.address, TokenInfo::struct_tag_for(token_code))
    }

    pub fn data_path_for(token_code: TokenCode) -> DataPath {
        AccessPath::resource_data_path(TokenInfo::struct_tag_for(token_code))
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}
