// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{write_op_value_view::WriteOpValueView, write_op_view::WriteOpView};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::access_path::AccessPath;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransactionOutputAction {
    pub access_path: AccessPath,
    pub action: WriteOpView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<WriteOpValueView>,
}
