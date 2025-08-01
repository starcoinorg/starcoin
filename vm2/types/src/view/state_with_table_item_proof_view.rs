// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{
    spare_merkle_proof_view::SparseMerkleProofView, state_with_proof_view::StateWithProofView,
    str_view::StrView,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StateWithTableItemProofView {
    pub state_proof: (StateWithProofView, HashValue),
    pub table_handle_proof: (Option<StrView<Vec<u8>>>, SparseMerkleProofView, HashValue),
    pub key_proof: (Option<StrView<Vec<u8>>>, SparseMerkleProofView, HashValue),
}
