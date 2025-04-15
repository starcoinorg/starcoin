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

// TODO(BobOng): [dual-vm] put it into definition file of StateWithTableItemProof
// impl StateWithTableItemProofView {
//     pub fn into_state_table_item_proof(self) -> StateWithTableItemProof {
//         self.into()
//     }
// }
//
// impl From<StateWithTableItemProof> for StateWithTableItemProofView {
//     fn from(state_table_item_proof: StateWithTableItemProof) -> Self {
//         Self {
//             state_proof: (
//                 state_table_item_proof.state_proof.0.into(),
//                 state_table_item_proof.state_proof.1,
//             ),
//             table_handle_proof: (
//                 state_table_item_proof.table_handle_proof.0.map(StrView),
//                 state_table_item_proof.table_handle_proof.1.into(),
//                 state_table_item_proof.table_handle_proof.2,
//             ),
//             key_proof: (
//                 state_table_item_proof.key_proof.0.map(StrView),
//                 state_table_item_proof.key_proof.1.into(),
//                 state_table_item_proof.key_proof.2,
//             ),
//         }
//     }
// }
//
// impl From<StateWithTableItemProofView> for StateWithTableItemProof {
//     fn from(view: StateWithTableItemProofView) -> Self {
//         let state_proof = (StateWithProof::from(view.state_proof.0), view.state_proof.1);
//         let table_handle_proof = (
//             view.table_handle_proof.0.map(|v| v.0),
//             SparseMerkleProof::from(view.table_handle_proof.1),
//             view.table_handle_proof.2,
//         );
//         let key_proof = (
//             view.key_proof.0.map(|v| v.0),
//             SparseMerkleProof::from(view.key_proof.1),
//             view.key_proof.2,
//         );
//         Self::new(state_proof, table_handle_proof, key_proof)
//     }
// }
