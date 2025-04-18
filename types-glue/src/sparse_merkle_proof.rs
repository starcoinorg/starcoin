// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use forkable_jellyfish_merkle::proof as VM1;
use starcoin_vm2_forkable_jellyfish_merkle::proof as VM2;

pub fn vm1_to_vm2(smp: VM1::SparseMerkleProof) -> VM2::SparseMerkleProof {
    VM2::SparseMerkleProof::new(smp.leaf, smp.siblings)
}

pub fn vm2_to_vm1(smp: VM2::SparseMerkleProof) -> VM1::SparseMerkleProof {
    VM1::SparseMerkleProof::new(smp.leaf, smp.siblings)
}
