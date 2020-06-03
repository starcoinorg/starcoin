// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod account_proof;

pub use account_proof::AccountProof;

// reuse libra in memory accumulator.
pub use libra_types::proof::accumulator::InMemoryAccumulator;
pub use libra_types::proof::{MerkleTreeInternalNode, SparseMerkleLeafNode};
