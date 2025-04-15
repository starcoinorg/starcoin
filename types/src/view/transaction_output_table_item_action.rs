// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{str_view::StrView, table_item_view::TableItemView, write_op_view::WriteOpView};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::write_set::WriteOp;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransactionOutputTableItemAction {
    pub table_item: TableItemView,
    pub action: WriteOpView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<StrView<Vec<u8>>>,
}

impl From<(TableItemView, WriteOp)> for TransactionOutputTableItemAction {
    fn from((table_item, op): (TableItemView, WriteOp)) -> Self {
        let (action, value) = match op {
            WriteOp::Deletion { .. } => (WriteOpView::Deletion, None),
            WriteOp::Modification { data, .. } | WriteOp::Creation { data, .. } => {
                (WriteOpView::Value, Some(StrView(data.to_vec())))
            }
        };

        Self {
            table_item,
            action,
            value,
        }
    }
}
