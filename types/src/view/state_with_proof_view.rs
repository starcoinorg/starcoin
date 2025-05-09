// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{spare_merkle_proof_view::SparseMerkleProofView, str_view::StrView};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StateWithProofView {
    pub state: Option<StrView<Vec<u8>>>,
    pub account_state: Option<StrView<Vec<u8>>>,
    pub account_proof: SparseMerkleProofView,
    pub account_state_proof: SparseMerkleProofView,
}
