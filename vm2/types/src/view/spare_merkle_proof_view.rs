// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use forkable_jellyfish_merkle::proof::SparseMerkleProof;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SparseMerkleProofView {
    /// This proof can be used to authenticate whether a given leaf exists in the tree or not.
    ///     - If this is `Some(HashValue, HashValue)`
    ///         - If the first `HashValue` equals requested key, this is an inclusion proof and the
    ///           second `HashValue` equals the hash of the corresponding account blob.
    ///         - Otherwise this is a non-inclusion proof. The first `HashValue` is the only key
    ///           that exists in the subtree and the second `HashValue` equals the hash of the
    ///           corresponding account blob.
    ///     - If this is `None`, this is also a non-inclusion proof which indicates the subtree is
    ///       empty.
    pub leaf: Option<(HashValue, HashValue)>,

    /// All siblings in this proof, including the default ones. Siblings are ordered from the bottom
    /// level to the root level.
    pub siblings: Vec<HashValue>,
}

impl From<SparseMerkleProof> for SparseMerkleProofView {
    fn from(origin: SparseMerkleProof) -> Self {
        Self {
            leaf: origin.leaf,
            siblings: origin.siblings,
        }
    }
}

impl From<SparseMerkleProofView> for SparseMerkleProof {
    fn from(origin: SparseMerkleProofView) -> Self {
        Self {
            leaf: origin.leaf,
            siblings: origin.siblings,
        }
    }
}
