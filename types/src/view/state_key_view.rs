// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{str_view::StrView, table_item_view::TableItemView};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::{
    access_path::AccessPath,
    state_store::state_key::{inner::StateKeyInner, StateKey},
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum StateKeyView {
    #[serde(rename = "access_path")]
    AccessPath(AccessPath),
    #[serde(rename = "table_item")]
    TableItem(TableItemView),
}

impl From<StateKey> for StateKeyView {
    fn from(state_key: StateKey) -> Self {
        match state_key.inner() {
            StateKeyInner::AccessPath(access_path) => Self::AccessPath(access_path.clone()),
            StateKeyInner::TableItem { handle, key } => Self::TableItem(TableItemView {
                handle: *handle,
                key: StrView::from(key.to_vec()),
            }),
            StateKeyInner::Raw(_) => todo!("not support raw key"),
        }
    }
}
