// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use crate::{account_config::constants::CORE_CODE_ADDRESS, move_resource::MoveResource};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::StructTag;
use serde::{Deserialize, Serialize};

/// The AutoAcceptToken resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct FrozenConfigBurnBlockNumberResource {
    block_number: u64,
}

impl FrozenConfigBurnBlockNumberResource {
    pub fn block_number(&self) -> u64 {
        self.block_number
    }

    pub fn struct_tag() -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: Identifier::new(FrozenConfigBurnBlockNumberResource::MODULE_NAME).unwrap(),
            module: Identifier::new(FrozenConfigBurnBlockNumberResource::STRUCT_NAME).unwrap(),
            type_params: vec![],
        }
    }

    pub fn access_path(address: AccountAddress) -> AccessPath {
        AccessPath::resource_access_path(address, Self::struct_tag())
    }
}

impl MoveResource for FrozenConfigBurnBlockNumberResource {
    const MODULE_NAME: &'static str = "FrozenConfigStrategy";
    const STRUCT_NAME: &'static str = "BurnBlockNumber";
}
