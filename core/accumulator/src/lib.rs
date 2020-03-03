// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use anyhow::{ensure, Error, Result};
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};

#[cfg(test)]
mod accumulator_test;

pub mod node;
pub mod node_index;

use crate::node::{InternalNode, ACCUMULATOR_PLACEHOLDER_HASH};
use crate::node_index::NodeIndex;
pub use node::AccumulatorNode;
use std::collections::HashMap;

pub type LeafCount = u64;
pub type NodeCount = u64;

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
    /// From leaves constructed accumulator
    fn from_leaves(&mut self, leaves: &[HashValue]) -> Self;
    /// Append leaves and return new root
    fn append(&mut self, leaves: &[HashValue]) -> Result<HashValue>;
    /// Get leaf hash by leaf index.
    fn get_leaf(&self, leaf_index: u64) -> Result<Option<HashValue>>;
    /// Get proof by leaf index.
    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>>;
    /// Get current accumulator tree root hash.
    fn root_hash(&self) -> HashValue;
    /// Get current accumulator tree number of leaves.
    fn num_leaves(&self) -> u64;
    /// Update current accumulator tree for rollback
    fn update(&mut self, leaf_index: u64, leaves: &[HashValue]) -> Result<HashValue>;
}

pub trait AccumulatorNodeReader {
    ///get node by node_index
    fn get(&self, index: NodeIndex) -> Result<Option<AccumulatorNode>>;
    ///get node by node hash
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>>;
}

pub trait AccumulatorNodeWriter {
    /// save node index
    fn save(&self, index: NodeIndex, hash: HashValue) -> Result<()>;
    /// save node
    fn save_node(&self, node: AccumulatorNode) -> Result<()>;
    ///delete node
    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<()>;
    ///delete larger index than one
    fn delete_larger_index(&self, index: u64, max_notes: u64) -> Result<()>;
}

pub trait AccumulatorNodeStore: AccumulatorNodeReader + AccumulatorNodeWriter {}

/// MerkleAccumulator is a accumulator algorithm implement and it is stateless.
pub struct MerkleAccumulator<'a, S> {
    frozen_subtree_roots: Vec<HashValue>,
    /// The total number of leaves in this accumulator.
    num_leaves: LeafCount,
    /// The total number of nodes in this accumulator.
    num_notes: NodeCount,
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
        num_notes: NodeCount,
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
            num_notes,
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
    ) -> u64 {
        // First just append the leaf.
        frozen_subtree_roots.push(leaf);

        // Next, merge the last two subtrees into one. If `num_existing_leaves` has N trailing
        // ones, the carry will happen N times.
        let num_trailing_ones = (!num_existing_leaves).trailing_zeros();
        let mut num_internal_nodes = 0u64;
        for i in 0..num_trailing_ones {
            let right_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let left_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let parent_hash =
                InternalNode::new(NodeIndex::new(i as u64), left_hash, right_hash).hash();
            frozen_subtree_roots.push(parent_hash);
            num_internal_nodes += 1;
        }
        num_internal_nodes
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
    ///delete node from leaf_index
    fn delete(&mut self, leaf_index: u64) -> Result<()> {
        ensure!(
            leaf_index < self.num_leaves as u64,
            "invalid leaf_index {}, num_leaves {}",
            leaf_index,
            self.num_leaves
        );
        //find deleting node by leaf_index
        let larger_nodes = self.get_larger_nodes_from_index(leaf_index).unwrap();
        //delete node and index
        self.node_store.delete_nodes(larger_nodes.clone());
        self.node_store
            .delete_larger_index(leaf_index, self.num_notes);

        // update self frozen_subtree_roots
        for hash in larger_nodes {
            let pos = self
                .frozen_subtree_roots
                .iter()
                .position(|x| *x == hash)
                .unwrap();
            self.frozen_subtree_roots.remove(pos);
        }
        //update self leaves number
        self.num_leaves = leaf_index - 1;

        //update root hash
        let node_index = NodeIndex::new(leaf_index);
        if node_index.is_left_child() {
            //if index is left, update root hash
            let new_root_index = NodeIndex::root_from_leaf_count(leaf_index);
            self.root_hash = self.node_store.get(new_root_index).unwrap().unwrap().hash();
        } else {
            self.root_hash = self.get_new_root_and_update_right_node(node_index).unwrap();
        };

        Ok(())
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

    ///get all nodes larger than index of leaf_node
    fn get_larger_nodes_from_index(&self, leaf_index: u64) -> Result<Vec<HashValue>> {
        let mut node_vec = vec![];
        for index in leaf_index..self.num_leaves {
            match self.node_store.get(NodeIndex::new(index)).unwrap() {
                Some(node) => node_vec.push(node.hash()),
                _ => {}
            }
        }
        Ok(node_vec)
    }

    ///get new root by right leaf index
    fn get_new_root_and_update_right_node(&self, leaf_index: NodeIndex) -> Result<HashValue> {
        let mut right_hash = *ACCUMULATOR_PLACEHOLDER_HASH;
        let mut right_index = leaf_index;
        //save current node
        self.node_store.save(leaf_index, right_hash);
        self.node_store
            .save_node(AccumulatorNode::new_leaf(leaf_index, right_hash));
        let mut new_root = right_hash;
        loop {
            //get sibling
            let left_node_index = right_index.sibling();
            match self.node_store.get(left_node_index).unwrap() {
                Some(node) => {
                    let left_hash = node.hash();
                    let parent_index = right_index.parent();
                    //set new root hash to parent node hash
                    let parent_node =
                        AccumulatorNode::new_internal(parent_index, left_hash, right_hash);
                    //save parent node
                    self.node_store.save_node(parent_node.clone());
                    new_root = parent_node.hash();
                    self.node_store.save(parent_index, new_root);
                    //for next loop
                    right_index = parent_node.index();
                    right_hash = new_root;
                }
                _ => {
                    break;
                }
            }
        }
        Ok(new_root)
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
    fn from_leaves(&mut self, leaves: &[HashValue]) -> Self {
        let mut frozen_subtree_roots = self.frozen_subtree_roots.clone();
        let mut num_leaves = self.num_leaves;
        let mut internal_notes = self.num_notes;
        for leaf in leaves {
            let temp_internal_notes =
                Self::append_one(&mut frozen_subtree_roots, num_leaves, *leaf);
            num_leaves += 1;
            internal_notes = internal_notes + temp_internal_notes;
        }
        self.num_leaves = num_leaves;
        self.num_notes = internal_notes + num_leaves;
        let accumulator = Self::new(
            frozen_subtree_roots,
            num_leaves,
            self.num_notes,
            self.node_store,
        )
        .expect("Appending leaves to a valid accumulator should create another valid accumulator.");
        self.root_hash = accumulator.root_hash;
        accumulator
    }

    fn append(&mut self, leaves: &[HashValue]) -> Result<HashValue, Error> {
        Ok(self.from_leaves(leaves).root_hash)
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

    fn num_leaves(&self) -> u64 {
        self.num_leaves
    }

    fn update(&mut self, leaf_index: u64, leaves: &[HashValue]) -> Result<HashValue, Error> {
        //ensure leaves is null
        ensure!(leaves.len() > 0, "invalid leaves len: {}", leaves.len());
        ensure!(
            leaf_index < self.num_leaves as u64,
            "invalid leaf_index {}, num_leaves {}",
            leaf_index,
            self.num_leaves
        );
        // delete larger nodes from index
        self.delete(leaf_index);
        // append new notes
        self.append(leaves)
    }
}

pub struct MockAccumulatorStore {
    index_store: HashMap<NodeIndex, HashValue>,
    node_store: HashMap<HashValue, AccumulatorNode>,
}

impl MockAccumulatorStore {
    pub fn new() -> Self {
        Self {
            index_store: HashMap::new(),
            node_store: HashMap::new(),
        }
    }
}

impl AccumulatorNodeStore for MockAccumulatorStore {}
impl AccumulatorNodeReader for MockAccumulatorStore {
    fn get(&self, index: NodeIndex) -> Result<Option<AccumulatorNode>, Error> {
        let node_hash = self.index_store.get(&index).unwrap();
        match self.node_store.get(&node_hash) {
            Some(node) => Ok(Some(node.clone())),
            None => bail!("get node is null"),
        }
    }

    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        match self.node_store.get(&hash) {
            Some(node) => Ok(Some(node.clone())),
            None => bail!("get node is null"),
        }
    }
}
impl AccumulatorNodeWriter for MockAccumulatorStore {
    fn save(&self, index: NodeIndex, hash: HashValue) -> Result<(), Error> {
        self.index_store.clone().insert(index, hash);
        Ok(())
    }

    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.node_store.clone().insert(node.hash(), node);
        Ok(())
    }

    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<(), Error> {
        for hash in node_hash_vec {
            self.node_store.clone().remove(&hash);
        }
        Ok(())
    }

    fn delete_larger_index(&self, from_index: u64, max_notes: u64) -> Result<(), Error> {
        ensure!(
            from_index <= max_notes,
            " invalid index form: {} to max notes:{}.",
            from_index,
            max_notes
        );
        for index in from_index..max_notes {
            self.index_store.clone().remove(&NodeIndex::new(index));
        }
        Ok(())
    }
}
