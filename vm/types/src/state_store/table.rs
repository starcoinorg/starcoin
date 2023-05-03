// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::TABLE_ADDRESS_LIST_LEN;
use move_core_types::{
    account_address::{AccountAddress, AccountAddressParseError},
    language_storage::TypeTag,
};
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, JsonSchema,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(proptest_derive::Arbitrary))]
pub struct TableHandle(pub AccountAddress);

impl TableHandle {
    pub fn size(&self) -> usize {
        std::mem::size_of_val(&self.0)
    }

    // XXX FIXME YSG add test
    pub fn get_idx(&self) -> usize {
        *self
            .0
            .into_bytes()
            .last()
            .expect("TableHandle array size > 0") as usize
            % TABLE_ADDRESS_LIST_LEN
    }
}

impl FromStr for TableHandle {
    type Err = AccountAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let handle = AccountAddress::from_str(s)?;
        Ok(Self(handle))
    }
}

impl From<move_table_extension::TableHandle> for TableHandle {
    fn from(hdl: move_table_extension::TableHandle) -> Self {
        Self(hdl.0)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct TableInfo {
    pub key_type: TypeTag,
    pub value_type: TypeTag,
}

impl TableInfo {
    pub fn new(key_type: TypeTag, value_type: TypeTag) -> Self {
        Self {
            key_type,
            value_type,
        }
    }
}
