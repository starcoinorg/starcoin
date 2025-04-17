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

// TODO(BobOng): [dual-vm] put it into definaction file of StateWithProof
// impl StateWithProofView {
//     pub fn into_state_proof(self) -> StateWithProof {
//         self.into()
//     }
// }
//
// impl From<StateWithProof> for StateWithProofView {
//     fn from(state_proof: StateWithProof) -> Self {
//         let state = state_proof.state.map(StrView);
//         Self {
//             state,
//             account_state: state_proof.proof.account_state.map(|b| StrView(b.into())),
//             account_proof: state_proof.proof.account_proof.into(),
//             account_state_proof: state_proof.proof.account_state_proof.into(),
//         }
//     }
// }
//
// impl From<StateWithProofView> for StateWithProof {
//     fn from(view: StateWithProofView) -> Self {
//         let state = view.state.map(|v| v.0);
//         let proof = StateProof::new(
//             view.account_state.map(|v| v.0),
//             view.account_proof.into(),
//             view.account_state_proof.into(),
//         );
//         Self::new(state, proof)
//     }
// }
