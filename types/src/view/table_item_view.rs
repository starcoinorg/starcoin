// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::str_view::StrView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::state_store::table::TableHandle;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "table_item")]
pub struct TableItemView {
    pub handle: TableHandle,
    pub key: StrView<Vec<u8>>,
}
