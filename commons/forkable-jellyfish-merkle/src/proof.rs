// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::blob::Blob;
use crate::node_type::{SparseMerkleInternalNode, SparseMerkleLeafNode};
use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::*;

/// A proof that can be used to authenticate an element in a Sparse Merkle Tree given trusted root
/// hash. For example, `TransactionInfoToAccountProof` can be constructed on top of this structure.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SparseMerkleProof {
    /// This proof can be used to authenticate whether a given leaf exists in the tree or not.
    ///     - If this is `Some(HashValue, HashValue)`
    ///         - If the first `HashValue` equals requested key, this is an inclusion proof and the
    ///           second `HashValue` equals the hash of the corresponding account blob.
    ///         - Otherwise this is a non-inclusion proof. The first `HashValue` is the only key
    ///           that exists in the subtree and the second `HashValue` equals the hash of the
    ///           corresponding blob.
    ///     - If this is `None`, this is also a non-inclusion proof which indicates the subtree is
    ///       empty.
    pub leaf: Option<(HashValue, HashValue)>,

    /// All siblings in this proof, including the default ones. Siblings are ordered from the bottom
    /// level to the root level.
    pub siblings: Vec<HashValue>,
}

impl SparseMerkleProof {
    /// Constructs a new `SparseMerkleProof` using leaf and a list of siblings.
    pub fn new(leaf: Option<(HashValue, HashValue)>, siblings: Vec<HashValue>) -> Self {
        SparseMerkleProof { leaf, siblings }
    }

    /// Returns the leaf node in this proof.
    pub fn leaf(&self) -> Option<(HashValue, HashValue)> {
        self.leaf
    }

    /// Returns the list of siblings in this proof.
    pub fn siblings(&self) -> &[HashValue] {
        &self.siblings
    }

    /// If `element_blob` is present, verifies an element whose key is `element_key` and value is
    /// `element_blob` exists in the Sparse Merkle Tree using the provided proof. Otherwise
    /// verifies the proof is a valid non-inclusion proof that shows this key doesn't exist in the
    /// tree.
    pub fn verify(
        &self,
        expected_root_hash: HashValue,
        element_key: HashValue,
        element_blob: Option<&Blob>,
    ) -> Result<()> {
        ensure!(
            self.siblings.len() <= HashValue::LENGTH_IN_BITS,
            "Sparse Merkle Tree proof has more than {} ({}) siblings.",
            HashValue::LENGTH_IN_BITS,
            self.siblings.len(),
        );

        match (element_blob, self.leaf) {
            (Some(blob), Some((proof_key, proof_value_hash))) => {
                // This is an inclusion proof, so the key and value hash provided in the proof
                // should match element_key and element_value_hash. `siblings` should prove the
                // route from the leaf node to the root.
                ensure!(
                    element_key == proof_key,
                    "Keys do not match. Key in proof: {:x}. Expected key: {:x}.",
                    proof_key,
                    element_key
                );
                let hash = blob.crypto_hash();
                ensure!(
                    hash == proof_value_hash,
                    "Value hashes do not match. Value hash in proof: {:x}. \
                     Expected value hash: {:x}",
                    proof_value_hash,
                    hash,
                );
            }
            (Some(_blob), None) => bail!("Expected inclusion proof. Found non-inclusion proof."),
            (None, Some((proof_key, _))) => {
                // This is a non-inclusion proof. The proof intends to show that if a leaf node
                // representing `element_key` is inserted, it will break a currently existing leaf
                // node represented by `proof_key` into a branch. `siblings` should prove the
                // route from that leaf node to the root.
                ensure!(
                    element_key != proof_key,
                    "Expected non-inclusion proof, but key exists in proof.",
                );
                ensure!(
                    element_key.common_prefix_bits_len(proof_key) >= self.siblings.len(),
                    "Key would not have ended up in the subtree where the provided key in proof \
                     is the only existing key, if it existed. So this is not a valid \
                     non-inclusion proof.",
                );
            }
            (None, None) => {
                // This is a non-inclusion proof. The proof intends to show that if a leaf node
                // representing `element_key` is inserted, it will show up at a currently empty
                // position. `sibling` should prove the route from this empty position to the root.
            }
        }

        let current_hash = self
            .leaf
            .map_or(*SPARSE_MERKLE_PLACEHOLDER_HASH, |(key, value_hash)| {
                SparseMerkleLeafNode::new(key, value_hash).crypto_hash()
            });
        let actual_root_hash = self
            .siblings
            .iter()
            .zip(
                element_key
                    .iter_bits()
                    .rev()
                    .skip(HashValue::LENGTH_IN_BITS - self.siblings.len()),
            )
            .fold(current_hash, |hash, (sibling_hash, bit)| {
                if bit {
                    SparseMerkleInternalNode::new(*sibling_hash, hash).crypto_hash()
                } else {
                    SparseMerkleInternalNode::new(hash, *sibling_hash).crypto_hash()
                }
            });
        ensure!(
            actual_root_hash == expected_root_hash,
            "Root hashes do not match. Actual root hash: {:x}. Expected root hash: {:x}.",
            actual_root_hash,
            expected_root_hash,
        );

        Ok(())
    }

    /// Update the leaf, and compute new root.
    /// Only available for non existence proof
    pub fn update_leaf(
        &mut self,
        element_key: HashValue,
        element_blob: &Blob,
    ) -> Result<HashValue> {
        let element_hash = element_blob.crypto_hash();

        let is_non_exists_proof = match self.leaf.as_ref() {
            None => true,
            Some((leaf_key, _leaf_value)) => &element_key != leaf_key,
        };
        ensure!(
            is_non_exists_proof,
            "Only non existence proof support update leaf, got element_key: {:?} leaf: {:?}",
            element_key,
            self.leaf,
        );

        let new_leaf_node = SparseMerkleLeafNode::new(element_key, element_hash);
        let current_hash = new_leaf_node.crypto_hash();
        if let Some(leaf_node) = self
            .leaf
            .as_ref()
            .map(|(leaf_key, leaf_value)| SparseMerkleLeafNode::new(*leaf_key, *leaf_value))
        {
            let mut new_siblings = vec![leaf_node.crypto_hash()];
            let prefix_len = leaf_node.key.common_prefix_bits_len(element_key);

            let place_holder_len = (prefix_len - self.siblings.len()) + 1;
            if place_holder_len > 0 {
                new_siblings.resize(place_holder_len, *SPARSE_MERKLE_PLACEHOLDER_HASH);
            }
            new_siblings.extend(self.siblings.iter());
            self.siblings = new_siblings;
        }
        let new_root_hash = self
            .siblings
            .iter()
            .zip(
                element_key
                    .iter_bits()
                    .rev()
                    .skip(HashValue::LENGTH_IN_BITS - self.siblings.len()),
            )
            .fold(current_hash, |hash, (sibling_hash, bit)| {
                if bit {
                    SparseMerkleInternalNode::new(*sibling_hash, hash).crypto_hash()
                } else {
                    SparseMerkleInternalNode::new(hash, *sibling_hash).crypto_hash()
                }
            });
        self.leaf = Some((element_key, element_hash));
        Ok(new_root_hash)
    }
}

/// A proof that can be used authenticate a range of consecutive leaves, from the leftmost leaf to
/// a certain one, in a sparse Merkle tree. For example, given the following sparse Merkle tree:
///
/// ```text
///                   root
///                  /     \
///                 /       \
///                /         \
///               o           o
///              / \         / \
///             a   o       o   h
///                / \     / \
///               o   d   e   X
///              / \         / \
///             b   c       f   g
/// ```
///
/// if the proof wants show that `[a, b, c, d, e]` exists in the tree, it would need the siblings
/// `X` and `h` on the right.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SparseMerkleRangeProof {
    /// The vector of siblings on the right of the path from root to last leaf. The ones near the
    /// bottom are at the beginning of the vector. In the above example, it's `[X, h]`.
    right_siblings: Vec<HashValue>,
}

impl SparseMerkleRangeProof {
    /// Constructs a new `SparseMerkleRangeProof`.
    pub fn new(right_siblings: Vec<HashValue>) -> Self {
        Self { right_siblings }
    }

    /// Returns the siblings.
    pub fn right_siblings(&self) -> &[HashValue] {
        &self.right_siblings
    }
}
