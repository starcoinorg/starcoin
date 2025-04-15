// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{
    transaction_output_view::TransactionOutputView, vm_status_explain_view::VmStatusExplainView,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DryRunOutputView {
    pub explained_status: VmStatusExplainView,
    #[serde(flatten)]
    pub txn_output: TransactionOutputView,
}
