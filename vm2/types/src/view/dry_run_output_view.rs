// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::transaction_output_view::TransactionOutputView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vm_status_translator::VmStatusExplainView;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DryRunOutputView {
    pub explained_status: VmStatusExplainView,
    #[serde(flatten)]
    pub txn_output: TransactionOutputView,
}
