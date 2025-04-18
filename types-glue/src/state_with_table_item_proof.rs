// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{sparse_merkle_proof, state_with_proof};
use starcoin_state_api::StateWithTableItemProof as StateWithTableItemProofVM1;
use starcoin_vm2_state_api::StateWithTableItemProof as StateWithTableItemProofVM2;

pub fn vm1_to_vm2(smtp: StateWithTableItemProofVM1) -> StateWithTableItemProofVM2 {
    StateWithTableItemProofVM2::new(
        (
            state_with_proof::vm1_to_vm2(smtp.state_proof.0),
            smtp.state_proof.1,
        ),
        (
            smtp.table_handle_proof.0,
            sparse_merkle_proof::vm1_to_vm2(smtp.table_handle_proof.1),
            smtp.table_handle_proof.2,
        ),
        (
            smtp.key_proof.0,
            sparse_merkle_proof::vm1_to_vm2(smtp.key_proof.1),
            smtp.key_proof.2,
        ),
    )
}

pub fn vm2_to_vm1(smtp: StateWithTableItemProofVM2) -> StateWithTableItemProofVM1 {
    StateWithTableItemProofVM1::new(
        (
            state_with_proof::vm2_to_vm1(smtp.state_proof.0),
            smtp.state_proof.1,
        ),
        (
            smtp.table_handle_proof.0,
            sparse_merkle_proof::vm2_to_vm1(smtp.table_handle_proof.1),
            smtp.table_handle_proof.2,
        ),
        (
            smtp.key_proof.0,
            sparse_merkle_proof::vm2_to_vm1(smtp.key_proof.1),
            smtp.key_proof.2,
        ),
    )
}
