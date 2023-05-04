// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::TABLE_ADDRESS_LIST_LEN;
use anyhow::format_err;
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

    pub fn get_idx(&self) -> Result<usize, anyhow::Error> {
        let binding = self.0.into_bytes();
        let val = binding.last();
        match val {
            Some(val) => Ok(*val as usize % TABLE_ADDRESS_LIST_LEN),
            _ => Err(format_err!("TableHandle array size > 0")),
        }
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

#[cfg(test)]
mod tests {
    use crate::state_store::table::TableHandle;
    use move_core_types::account_address::AccountAddress;

    #[test]
    fn test_get_idx() -> Result<(), anyhow::Error> {
        let hdl1 = TableHandle(AccountAddress::ZERO);
        let idx1 = hdl1.get_idx()?;
        assert_eq!(idx1, 0);

        let hdl2 = TableHandle(AccountAddress::ONE);
        let idx2 = hdl2.get_idx()?;
        assert_eq!(idx2, 1);

        let hdl3 = TableHandle(AccountAddress::TWO);
        let idx3 = hdl3.get_idx()?;
        assert_eq!(idx3, 2);
        Ok(())
    }
}
