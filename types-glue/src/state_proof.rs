// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sparse_merkle_proof;
use starcoin_state_api as VM1;
use starcoin_vm2_state_api as VM2;

pub fn vm1_to_vm2(proof: VM1::StateProof) -> VM2::StateProof {
    VM2::StateProof::new(
        proof.account_state.map(|blob| blob.as_ref().to_vec()),
        sparse_merkle_proof::vm1_to_vm2(proof.account_proof),
        sparse_merkle_proof::vm1_to_vm2(proof.account_state_proof),
    )
}

pub fn vm2_to_vm1(proof: VM2::StateProof) -> VM1::StateProof {
    VM1::StateProof::new(
        proof.account_state.map(|blob| blob.as_ref().to_vec()),
        sparse_merkle_proof::vm2_to_vm1(proof.account_proof),
        sparse_merkle_proof::vm2_to_vm1(proof.account_state_proof),
    )
}
