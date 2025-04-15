// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::function_arg_type_view::TypeTagView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::state_store::table::TableInfo;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "table_info")]
pub struct TableInfoView {
    key_type: TypeTagView,
    value_type: TypeTagView,
}

impl From<TableInfo> for TableInfoView {
    fn from(value: TableInfo) -> Self {
        Self {
            key_type: value.key_type.into(),
            value_type: value.value_type.into(),
        }
    }
}

impl From<TableInfoView> for TableInfo {
    fn from(value: TableInfoView) -> Self {
        Self {
            key_type: value.key_type.0,
            value_type: value.value_type.0,
        }
    }
}
