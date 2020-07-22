// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_config::constants::{
        association_address, type_tag_for_currency_code, CORE_CODE_ADDRESS,
    },
    event::EventHandle,
};
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use move_core_types::{
    identifier::Identifier,
    language_storage::{ResourceKey, StructTag},
    move_resource::MoveResource,
};
use serde::{Deserialize, Serialize};

/// Struct that represents a TokenInfo resource
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfoResource {
    total_value: u128,
    scaling_factor: u64,
    fractional_part: u64,
    mint_events: EventHandle,
    burn_events: EventHandle,
}

impl MoveResource for TokenInfoResource {
    const MODULE_NAME: &'static str = "Token";
    const STRUCT_NAME: &'static str = "TokenInfo";
}

impl TokenInfoResource {
    pub fn scaling_factor(&self) -> u64 {
        self.scaling_factor
    }

    pub fn fractional_part(&self) -> u64 {
        self.fractional_part
    }

    pub fn struct_tag_for(
        token_module_address: AccountAddress,
        token_name: Identifier,
    ) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: TokenInfoResource::module_identifier(),
            name: TokenInfoResource::struct_identifier(),
            type_params: vec![type_tag_for_currency_code(
                Some(token_module_address),
                token_name,
            )],
        }
    }

    pub fn resource_path_for(
        currency_module_address: AccountAddress,
        currency_code: Identifier,
    ) -> AccessPath {
        let resource_address = if currency_module_address == CORE_CODE_ADDRESS {
            association_address()
        } else {
            currency_module_address
        };
        let resource_key = ResourceKey::new(
            resource_address,
            TokenInfoResource::struct_tag_for(currency_module_address, currency_code),
        );
        AccessPath::resource_access_path(&resource_key)
    }

    pub fn access_path_for(
        currency_module_address: AccountAddress,
        currency_code: Identifier,
    ) -> Vec<u8> {
        AccessPath::resource_access_vec(&TokenInfoResource::struct_tag_for(
            currency_module_address,
            currency_code,
        ))
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        scs::from_bytes(bytes).map_err(Into::into)
    }
}
