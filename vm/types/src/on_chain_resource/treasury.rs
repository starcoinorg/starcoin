// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use crate::account_address::AccountAddress;
use crate::account_config::CORE_CODE_ADDRESS;
use crate::event::EventHandle;
use crate::language_storage::{StructTag, TypeTag};
use crate::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

/// A Rust representation of a Treasury resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct Treasury {
    pub balance: u128,
    /// event handle for treasury withdraw event
    pub withdraw_events: EventHandle,
    /// event handle for treasury deposit event
    pub deposit_events: EventHandle,
}

impl MoveResource for Treasury {
    const MODULE_NAME: &'static str = "Treasury";
    const STRUCT_NAME: &'static str = "Treasury";
}

impl Treasury {
    pub fn struct_tag_for(token_type_tag: StructTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Self::module_identifier(),
            name: Self::struct_identifier(),
            type_params: vec![TypeTag::Struct(token_type_tag)],
        }
    }

    pub fn resource_path_for(token_type_tag: StructTag) -> AccessPath {
        AccessPath::resource_access_path(
            token_type_tag.address,
            Self::struct_tag_for(token_type_tag),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinearWithdrawCapability {
    pub total: u128,
    pub withdraw: u128,
    pub start_time: u64,
    pub period: u64,
}

impl MoveResource for LinearWithdrawCapability {
    const MODULE_NAME: &'static str = "Treasury";
    const STRUCT_NAME: &'static str = "LinearWithdrawCapability";
}

impl LinearWithdrawCapability {
    pub fn struct_tag_for(token_type_tag: StructTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Self::module_identifier(),
            name: Self::struct_identifier(),
            type_params: vec![TypeTag::Struct(token_type_tag)],
        }
    }

    pub fn resource_path_for(address: AccountAddress, token_type_tag: StructTag) -> AccessPath {
        AccessPath::resource_access_path(address, Self::struct_tag_for(token_type_tag))
    }
}
