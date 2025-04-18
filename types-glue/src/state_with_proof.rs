// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_proof;
use starcoin_state_api::StateWithProof as StateWithProofVM1;
use starcoin_vm2_state_api::StateWithProof as StateWithProofVM2;

pub fn vm1_to_vm2(proof: StateWithProofVM1) -> StateWithProofVM2 {
    StateWithProofVM2::new(proof.state, state_proof::vm1_to_vm2(proof.proof))
}

pub fn vm2_to_vm1(proof: StateWithProofVM2) -> StateWithProofVM1 {
    StateWithProofVM1::new(proof.state, state_proof::vm2_to_vm1(proof.proof))
}
