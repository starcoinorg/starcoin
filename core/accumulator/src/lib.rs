// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use anyhow::{ensure, Error, Result};
use crypto::{hash::CryptoHash, HashValue};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;

pub mod node;
pub mod node_index;

use crate::node::{InternalNode, ACCUMULATOR_PLACEHOLDER_HASH};
use crate::node_index::NodeIndex;
pub use node::AccumulatorNode;

pub type LeafCount = u64;

pub const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;
pub const MAX_ACCUMULATOR_LEAVES: LeafCount = 1 << MAX_ACCUMULATOR_PROOF_DEPTH;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccumulatorProof {
    /// All siblings in this proof, including the default ones. Siblings are ordered from the bottom
    /// level to the root level.
    siblings: Vec<HashValue>,
}

impl AccumulatorProof {
    /// Constructs a new `AccumulatorProof` using a list of siblings.
    pub fn new(siblings: Vec<HashValue>) -> Self {
        AccumulatorProof { siblings }
    }

    /// Returns the list of siblings in this proof.
    pub fn siblings(&self) -> &[HashValue] {
        &self.siblings
    }

    /// Verifies an element whose hash is `element_hash` exists in
    /// the accumulator whose root hash is `expected_root_hash` using the provided proof.
    pub fn verify(
        &self,
        expected_root_hash: HashValue,
        element_hash: HashValue,
        element_index: u64,
    ) -> Result<()> {
        ensure!(
            self.siblings.len() <= MAX_ACCUMULATOR_PROOF_DEPTH,
            "Accumulator proof has more than {} ({}) siblings.",
            MAX_ACCUMULATOR_PROOF_DEPTH,
            self.siblings.len()
        );

        let actual_root_hash = self
            .siblings
            .iter()
            .fold(
                (element_hash, element_index),
                // `index` denotes the index of the ancestor of the element at the current level.
                |(hash, index), sibling_hash| {
                    (
                        if index % 2 == 0 {
                            // the current node is a left child.
                            InternalNode::new(NodeIndex::new(index), hash, *sibling_hash).hash()
                        } else {
                            // the current node is a right child.
                            InternalNode::new(NodeIndex::new(index), *sibling_hash, hash).hash()
                        },
                        // The index of the parent at its level.
                        index / 2,
                    )
                },
            )
            .0;
        ensure!(
            actual_root_hash == expected_root_hash,
            "Root hashes do not match. Actual root hash: {:x}. Expected root hash: {:x}.",
            actual_root_hash,
            expected_root_hash
        );

        Ok(())
    }
}
/// accumulator method define
pub trait Accumulator {
    /// Append leaves and return new root
    fn append(&self, leaves: &[HashValue]) -> Result<HashValue>;
    /// Get leaf hash by leaf index.
    fn get_leaf(&self, leaf_index: u64) -> Result<Option<HashValue>>;

    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>>;

    fn root_hash(&self) -> HashValue;
}

pub trait AccumulatorNodeReader {
    ///get node by node_index
    fn get(&self, index: NodeIndex) -> Result<Option<AccumulatorNode>>;
    ///get node by node hash
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>>;
}

pub trait AccumulatorNodeWriter {
    fn save(&self, index: NodeIndex, hash: HashValue) -> Result<()>;
    fn save_node(&self, node: AccumulatorNode) -> Result<()>;
}

pub trait AccumulatorNodeStore: AccumulatorNodeReader + AccumulatorNodeWriter {}

/// MerkleAccumulator is a accumulator algorithm implement and it is stateless.
pub struct MerkleAccumulator<'a, S> {
    frozen_subtree_roots: Vec<HashValue>,
    /// The total number of leaves in this accumulator.
    num_leaves: LeafCount,
    /// The root hash of this accumulator.
    root_hash: HashValue,
    node_store: &'a S,
}

impl<'a, S> MerkleAccumulator<'a, S>
where
    S: AccumulatorNodeStore,
{
    pub fn new(
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: LeafCount,
        node_store: &'a S,
    ) -> Result<Self> {
        ensure!(
            frozen_subtree_roots.len() == num_leaves.count_ones() as usize,
            "The number of frozen subtrees does not match the number of leaves. \
             frozen_subtree_roots.len(): {}. num_leaves: {}.",
            frozen_subtree_roots.len(),
            num_leaves,
        );

        let root_hash = Self::compute_root_hash(&frozen_subtree_roots, num_leaves);

        Ok(Self {
            frozen_subtree_roots,
            num_leaves,
            root_hash,
            node_store,
        })
    }
    /// Appends one leaf. This will update `frozen_subtree_roots` to store new frozen root nodes
    /// and remove old nodes if they are now part of a larger frozen subtree.
    fn append_one(
        frozen_subtree_roots: &mut Vec<HashValue>,
        num_existing_leaves: LeafCount,
        leaf: HashValue,
    ) {
        // First just append the leaf.
        frozen_subtree_roots.push(leaf);

        // Next, merge the last two subtrees into one. If `num_existing_leaves` has N trailing
        // ones, the carry will happen N times.
        let num_trailing_ones = (!num_existing_leaves).trailing_zeros();

        for i in 0..num_trailing_ones {
            let right_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let left_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let parent_hash =
                InternalNode::new(NodeIndex::new(i as u64), left_hash, right_hash).hash();
            frozen_subtree_roots.push(parent_hash);
        }
    }

    /// Computes the root hash of an accumulator given the frozen subtree roots and the number of
    /// leaves in this accumulator.
    fn compute_root_hash(frozen_subtree_roots: &[HashValue], num_leaves: LeafCount) -> HashValue {
        match frozen_subtree_roots.len() {
            0 => return *ACCUMULATOR_PLACEHOLDER_HASH,
            1 => return frozen_subtree_roots[0],
            _ => (),
        }

        // The trailing zeros do not matter since anything below the lowest frozen subtree is
        // already represented by the subtree roots.
        let mut bitmap = num_leaves >> num_leaves.trailing_zeros();
        let mut current_hash = *ACCUMULATOR_PLACEHOLDER_HASH;
        let mut frozen_subtree_iter = frozen_subtree_roots.iter().rev();
        let mut index = 0u64;

        while bitmap > 0 {
            let node_index = NodeIndex::new(index);
            current_hash = if bitmap & 1 != 0 {
                InternalNode::new(
                    node_index,
                    *frozen_subtree_iter
                        .next()
                        .expect("This frozen subtree should exist."),
                    current_hash,
                )
            } else {
                InternalNode::new(node_index, current_hash, *ACCUMULATOR_PLACEHOLDER_HASH)
            }
            .hash();
            bitmap >>= 1;
            index += 1;
        }

        current_hash
    }
    /// filter function can be applied to filter out certain siblings.
    fn get_siblings(
        &self,
        leaf_index: u64,
        filter: impl Fn(NodeIndex) -> bool,
    ) -> Result<Vec<HashValue>> {
        let root_pos = NodeIndex::root_from_leaf_count(self.num_leaves);
        let siblings = NodeIndex::from_leaf_index(leaf_index)
            .iter_ancestor_sibling()
            .take(root_pos.level() as usize)
            .filter_map(|p| {
                if filter(p) {
                    Some(self.get_node_hash(p))
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(siblings)
    }

    fn rightmost_leaf_index(&self) -> u64 {
        (self.num_leaves - 1) as u64
    }

    fn get_node_hash(&self, node_index: NodeIndex) -> Result<HashValue> {
        let node = self.node_store.get(node_index).unwrap();
        match node {
            Some(acc_node) => match acc_node {
                AccumulatorNode::Internal(inter) => Ok(inter.hash()),
                AccumulatorNode::Leaf(leaf) => Ok(leaf.value()),
                AccumulatorNode::Empty => Ok(*ACCUMULATOR_PLACEHOLDER_HASH),
            },
            None => bail!("node is null: {:?}", node_index),
        }
    }
}

impl<'a, S> Accumulator for MerkleAccumulator<'a, S>
where
    S: AccumulatorNodeStore,
{
    fn append(&self, leaves: &[HashValue]) -> Result<HashValue, Error> {
        let mut frozen_subtree_roots = self.frozen_subtree_roots.clone();
        let mut num_leaves = self.num_leaves;
        for leaf in leaves {
            Self::append_one(&mut frozen_subtree_roots, num_leaves, *leaf);
            num_leaves += 1;
        }

        Ok(Self::new(frozen_subtree_roots, num_leaves, self.node_store)
            .expect(
                "Appending leaves to a valid accumulator should create another valid accumulator.",
            )
            .root_hash)
    }

    fn get_leaf(&self, leaf_index: u64) -> Result<Option<HashValue>, Error> {
        Ok(Some(
            self.get_node_hash(NodeIndex::new(leaf_index)).unwrap(),
        ))
    }

    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>, Error> {
        ensure!(
            leaf_index < self.num_leaves as u64,
            "invalid leaf_index {}, num_leaves {}",
            leaf_index,
            self.num_leaves
        );

        let siblings = self.get_siblings(leaf_index, |_p| true)?;
        Ok(Some(AccumulatorProof::new(siblings)))
    }

    fn root_hash(&self) -> HashValue {
        self.root_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulator() {}
}
