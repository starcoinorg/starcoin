// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{
    str_view::StrView, table_item_view::TableItemView,
    transaction_event_view::TransactionEventView,
    transaction_output_action::TransactionOutputAction,
    transaction_output_table_item_action::TransactionOutputTableItemAction,
    transaction_status_view::TransactionStatusView, write_op_value_view::WriteOpValueView,
    write_op_view::WriteOpView,
};
use bytes::Bytes;
use move_core_types::language_storage::StructTag;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::{
    access_path::{AccessPath, DataPath},
    state_store::state_key::inner::StateKeyInner,
    transaction::TransactionOutput,
    write_set::WriteOp,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransactionOutputView {
    pub status: TransactionStatusView,
    pub gas_used: StrView<u64>,
    pub write_set: Vec<TransactionOutputAction>,
    pub events: Vec<TransactionEventView>,
    pub table_item_write_set: Vec<TransactionOutputTableItemAction>,
}

fn merge_ap_write_set(
    output: &mut Vec<TransactionOutputAction>,
    access_path: AccessPath,
    op: WriteOp,
) {
    match op {
        WriteOp::Deletion { .. } => output.push(TransactionOutputAction {
            access_path,
            action: WriteOpView::Deletion,
            value: None,
        }),
        WriteOp::Modification { data, .. } | WriteOp::Creation { data, .. } => {
            match access_path.path {
                DataPath::Resource(_) => output.push(TransactionOutputAction {
                    access_path,
                    action: WriteOpView::Value,
                    value: Some(WriteOpValueView::Resource(data.to_vec().into())),
                }),
                // Parsing resources in group and expanding them into individual actions.
                DataPath::ResourceGroup(_) => {
                    let group_data: BTreeMap<StructTag, Bytes> =
                        bcs_ext::from_bytes(&data).expect("resource group data must be valid");
                    for (struct_tag, data) in group_data {
                        let resource_ap =
                            AccessPath::resource_access_path(access_path.address, struct_tag);
                        output.push(TransactionOutputAction {
                            access_path: resource_ap,
                            action: WriteOpView::Value,
                            value: Some(WriteOpValueView::Resource(data.to_vec().into())),
                        });
                    }
                }
                DataPath::Code(_) => output.push(TransactionOutputAction {
                    access_path,
                    action: WriteOpView::Value,
                    value: Some(WriteOpValueView::Code(data.to_vec().into())),
                }),
            }
        }
    };
}

impl From<TransactionOutput> for TransactionOutputView {
    fn from(txn_output: TransactionOutput) -> Self {
        let (write_set, events, gas_used, status, _) = txn_output.into_inner();
        let mut access_write_set = vec![];
        let mut table_item_write_set = vec![];
        for (state_key, op) in write_set {
            match state_key.inner() {
                StateKeyInner::AccessPath(access_path) => {
                    access_write_set.push((access_path.clone(), op));
                }
                StateKeyInner::TableItem { handle, key } => {
                    let table_item = TableItemView {
                        handle: *handle,
                        key: StrView::from(key.to_vec()),
                    };
                    table_item_write_set.push((table_item, op));
                }
                StateKeyInner::Raw(_) => todo!("not support raw key"),
            }
        }
        let write_set =
            access_write_set
                .into_iter()
                .fold(Vec::new(), |mut acc, (access_path, op)| {
                    merge_ap_write_set(&mut acc, access_path, op);
                    acc
                });

        Self {
            events: events.into_iter().map(Into::into).collect(),
            gas_used: gas_used.into(),
            status: status.into(),
            write_set,
            table_item_write_set: table_item_write_set
                .into_iter()
                .map(TransactionOutputTableItemAction::from)
                .collect(),
        }
    }
}
