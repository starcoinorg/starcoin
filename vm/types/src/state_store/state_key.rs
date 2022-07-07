// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Hash, JsonSchema,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(proptest_derive::Arbitrary))]
pub enum StateKey {
    AccessPath(AccessPath),
    TableItem {
        #[schemars(with = "String")]
        handle: u128,
        #[serde(with = "serde_bytes")]
        #[schemars(with = "String")]
        key: Vec<u8>,
    },
}

impl StateKey {
    pub fn table_item(handle: u128, key: Vec<u8>) -> Self {
        StateKey::TableItem { handle, key }
    }
}
