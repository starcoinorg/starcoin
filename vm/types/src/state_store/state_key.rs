// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use crate::state_store::table::TableHandle;
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Hash, JsonSchema,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(proptest_derive::Arbitrary))]
pub struct TableItem {
    pub handle: TableHandle,
    #[serde(with = "serde_bytes")]
    #[schemars(with = "String")]
    pub key: Vec<u8>,
}

#[derive(
    Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Hash, JsonSchema,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(proptest_derive::Arbitrary))]
pub enum StateKey {
    AccessPath(AccessPath),
    TableItem(TableItem),
}

impl StateKey {
    pub fn table_item(handle: TableHandle, key: Vec<u8>) -> Self {
        StateKey::TableItem(TableItem { handle, key })
    }
}
