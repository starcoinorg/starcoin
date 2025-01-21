// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    ident_str,
    identifier::{IdentStr, Identifier},
    language_storage::{StructTag, TypeTag, CORE_CODE_ADDRESS},
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

use crate::access_path::{AccessPath, DataPath};
use crate::account_config::token_code::TokenCode;
use crate::{event::EventHandle, utility_coin::STARCOIN_COIN_TYPE};

/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct CoinStoreResource {
    coin: u64,
    frozen: bool,
    deposit_events: EventHandle,
    withdraw_events: EventHandle,
}

impl CoinStoreResource {
    pub fn new(
        coin: u64,
        frozen: bool,
        deposit_events: EventHandle,
        withdraw_events: EventHandle,
    ) -> Self {
        Self {
            coin,
            frozen,
            deposit_events,
            withdraw_events,
        }
    }

    pub fn coin(&self) -> u64 {
        self.coin
    }

    pub fn frozen(&self) -> bool {
        self.frozen
    }

    pub fn deposit_events(&self) -> &EventHandle {
        &self.deposit_events
    }

    pub fn withdraw_events(&self) -> &EventHandle {
        &self.withdraw_events
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

impl MoveStructType for CoinStoreResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("coin");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CoinStore");

    fn type_args() -> Vec<TypeTag> {
        vec![STARCOIN_COIN_TYPE.clone()]
    }
}

impl MoveResource for CoinStoreResource {}
