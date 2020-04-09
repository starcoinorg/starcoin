// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use anyhow::{ensure, Error, Result};
use logger::prelude::*;
use mirai_annotations::*;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

#[cfg(test)]
mod accumulator_test;

pub mod node;
pub mod node_index;

use crate::node::{InternalNode, ACCUMULATOR_PLACEHOLDER_HASH};
use crate::node_index::{FrozenSubTreeIterator, NodeIndex, NodeStoreIndex};
pub use node::AccumulatorNode;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

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
    /// Append leaves and return new root
    fn append(&self, leaves: &[HashValue]) -> Result<(HashValue, u64), Error>;
    /// Append leaves and return new root, but not persistence
    fn append_only_cache(&self, leaves: &[HashValue]) -> Result<(HashValue, u64), Error>;
    /// Get leaf hash by leaf index.
    fn get_leaf(&self, leaf_index: u64) -> Result<Option<HashValue>>;
    /// Get proof by leaf index.
    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>>;
    /// Get current accumulator tree root hash.
    fn root_hash(&self) -> HashValue;
    /// Get current accumulator tree number of leaves.
    fn num_leaves(&self) -> u64;
    /// Get current accumulator tree number of nodes.
    fn num_nodes(&self) -> u64;
    /// Update current accumulator tree for rollback
    fn update(&self, leaf_index: u64, leaves: &[HashValue]) -> Result<(HashValue, u64), Error>;

    fn get_frozen_subtree_roots(&self) -> Result<Vec<HashValue>>;
}

pub trait AccumulatorReader {
    ///get node by node_index
    fn get(&self, index: NodeStoreIndex) -> Result<Option<AccumulatorNode>>;
    ///get node by node hash
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>>;
}

pub trait AccumulatorWriter {
    /// save node index
    fn save(&self, index: NodeStoreIndex, hash: HashValue) -> Result<()>;
    /// save node
    fn save_node(&self, node: AccumulatorNode) -> Result<()>;
    ///delete node
    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<()>;
    ///delete indexes
    fn delete_nodes_index(&self, index_vec: Vec<NodeStoreIndex>) -> Result<()>;
}

pub trait AccumulatorTreeStore: AccumulatorReader + AccumulatorWriter {}

/// MerkleAccumulator is a accumulator algorithm implement and it is stateless.
pub struct MerkleAccumulator {
    cache: Mutex<AccumulatorCache>,
    #[allow(dead_code)]
    node_store: Arc<dyn AccumulatorTreeStore>,
}

pub struct AccumulatorCache {
    /// Accumulator id
    id: HashValue,
    /// forzen subtree roots hashes.
    frozen_subtree_roots: RefCell<Vec<HashValue>>,
    /// index cache for node_index map to hash value.
    index_cache: RefCell<HashMap<NodeIndex, HashValue>>,
    /// The total number of leaves in this accumulator.
    num_leaves: LeafCount,
    /// The total number of nodes in this accumulator.
    num_nodes: NodeCount,
    /// The root hash of this accumulator.
    root_hash: HashValue,

    node_store: Arc<dyn AccumulatorTreeStore>,
}

impl AccumulatorCache {
    pub fn new(
        accumulator_id: HashValue,
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: LeafCount,
        num_nodes: NodeCount,
        root_hash: HashValue,
        internal_vec: Vec<HashValue>,
        node_store: Arc<dyn AccumulatorTreeStore>,
    ) -> Self {
        info!("accumulator cache new: {:?}", accumulator_id.short_str());
        Self {
            id: accumulator_id,
            frozen_subtree_roots: RefCell::new(frozen_subtree_roots.clone()),
            index_cache: RefCell::new(
                Self::aggregate_cache(
                    root_hash,
                    internal_vec,
                    frozen_subtree_roots,
                    node_store.clone(),
                )
                .unwrap(),
            ),
            num_leaves,
            num_nodes,
            root_hash,
            node_store,
        }
    }

    ///append from multiple leaves
    fn append_leaves(
        &mut self,
        new_leaves: &[HashValue],
    ) -> Result<(HashValue, Vec<AccumulatorNode>)> {
        // Deal with the case where new_leaves is empty
        if new_leaves.is_empty() {
            if self.num_leaves == 0 {
                return Ok((*ACCUMULATOR_PLACEHOLDER_HASH, Vec::new()));
            } else {
                return Ok((self.root_hash, Vec::new()));
            }
        }

        let num_new_leaves = new_leaves.len();
        let last_new_leaf_count = self.num_leaves + num_new_leaves as LeafCount;
        let mut new_num_nodes = self.num_nodes;
        let root_level = NodeIndex::root_level_from_leaf_count(last_new_leaf_count);
        let mut to_freeze = Vec::with_capacity(Self::max_to_freeze(num_new_leaves, root_level));
        // Iterate over the new leaves, adding them to to_freeze and then adding any frozen parents
        // when right children are encountered.  This has the effect of creating frozen nodes in
        // perfect post-order, which can be used as a strictly increasing append only index for
        // the underlying storage.
        //
        // We will track newly created left siblings while iterating so we can pair them with their
        // right sibling, if and when it becomes frozen.  If the frozen left sibling is not created
        // in this iteration, it must already exist in storage.
        let mut left_siblings: Vec<(_, _)> = Vec::new();
        for (leaf_offset, leaf) in new_leaves.iter().enumerate() {
            let leaf_pos = NodeIndex::from_leaf_index(self.num_leaves + leaf_offset as LeafCount);
            let mut hash = *leaf;
            to_freeze.push(AccumulatorNode::new_leaf(leaf_pos, hash));
            debug!(
                "{:?} insert leaf cache: {:?}",
                self.id.short_str(),
                leaf_pos
            );
            self.index_cache.borrow_mut().insert(leaf_pos, hash);
            new_num_nodes += 1;
            let mut pos = leaf_pos;
            while pos.is_right_child() {
                #[allow(unused_assignments)]
                let mut internal_node = AccumulatorNode::Empty;
                let sibling = pos.sibling();

                hash = match left_siblings.pop() {
                    Some((x, left_hash)) => {
                        assert_eq!(x, sibling);
                        internal_node =
                            AccumulatorNode::new_internal(pos.parent(), left_hash, hash);
                        internal_node.hash()
                        // Self::hash_internal_node(left_hash, hash)
                    }
                    None => {
                        internal_node = AccumulatorNode::new_internal(
                            pos.parent(),
                            self.get_index(sibling).unwrap(),
                            hash,
                        );
                        internal_node.hash()
                    }
                };
                pos = pos.parent();
                to_freeze.push(internal_node);
                debug!("{:?} insert internal cache: {:?}", self.id.short_str(), pos);
                self.index_cache.borrow_mut().insert(pos, hash);
                new_num_nodes += 1;
            }
            // The node remaining must be a left child, possibly the root of a complete binary tree.
            left_siblings.push((pos, hash));
        }

        // Now reconstruct the final root hash by walking up to root level and adding
        // placeholder hash nodes as needed on the right, and left siblings that have either
        // been newly created or read from storage.
        let (mut pos, mut hash) = left_siblings.pop().expect("Must have at least one node");
        for _ in pos.level()..root_level as u32 {
            hash = if pos.is_left_child() {
                AccumulatorNode::new_internal(pos.parent(), hash, *ACCUMULATOR_PLACEHOLDER_HASH)
                    .hash()
            } else {
                let sibling = pos.sibling();
                match left_siblings.pop() {
                    Some((x, left_hash)) => {
                        assert_eq!(x, sibling);
                        AccumulatorNode::new_internal(pos.parent(), left_hash, hash).hash()
                    }
                    None => AccumulatorNode::new_internal(
                        pos.parent(),
                        self.get_index(sibling).unwrap(),
                        hash,
                    )
                    .hash(),
                }
            };
            pos = pos.parent();
        }
        assert!(left_siblings.is_empty());

        self.root_hash = hash;
        self.frozen_subtree_roots = RefCell::new(Self::get_vec_hash(to_freeze.clone()).unwrap());
        self.num_leaves = last_new_leaf_count;
        self.num_nodes = new_num_nodes;

        Ok((hash, to_freeze))
    }

    fn _get_frozen_subtree_roots(&self) -> Result<Vec<HashValue>> {
        Ok(self.frozen_subtree_roots.borrow().to_vec())
    }

    fn get_node_store_index_vec(&self, index_vec: Vec<NodeIndex>) -> Vec<NodeStoreIndex> {
        index_vec
            .into_iter()
            .map(|v| NodeStoreIndex::new(self.id, v))
            .collect::<Vec<NodeStoreIndex>>()
    }

    fn get_index(&self, index: NodeIndex) -> Result<HashValue> {
        match self.index_cache.borrow().get(&index) {
            Some(hash) => Ok(*hash),
            None => bail!(
                "{:?} get index from cache error: {:?}",
                self.id.short_str(),
                index
            ),
        }
    }

    fn aggregate_cache(
        root_hash: HashValue,
        internal_vec: Vec<HashValue>,
        frozen_node_vec: Vec<HashValue>,
        store: Arc<dyn AccumulatorTreeStore>,
    ) -> Result<HashMap<NodeIndex, HashValue>> {
        let mut cache_map = HashMap::new();
        // get root hash
        let root_map = Self::restore_index_cache(root_hash, store.clone()).unwrap();
        cache_map.extend(root_map.iter());
        // restore from frozen node
        for hash in frozen_node_vec {
            if hash != root_hash {
                let tmp_map = Self::restore_index_cache(hash, store.clone()).unwrap();
                cache_map.extend(tmp_map.iter());
            }
        }
        // restore from internal node
        for hash in internal_vec {
            match store.get_node(hash) {
                Ok(Some(node)) => {
                    cache_map.insert(node.index(), node.hash());
                }
                _ => {}
            }
        }
        Ok(cache_map)
    }

    fn restore_index_cache(
        root_hash: HashValue,
        store: Arc<dyn AccumulatorTreeStore>,
    ) -> Result<HashMap<NodeIndex, HashValue>> {
        let mut cache_map = HashMap::new();
        info!("restore index cache, root:{:?}", root_hash.short_str());
        if root_hash != *ACCUMULATOR_PLACEHOLDER_HASH {
            //get node from storage
            match store.get_node(root_hash) {
                Ok(Some(node)) => {
                    //save index to cache
                    cache_map.insert(node.index(), node.hash());
                    match node {
                        AccumulatorNode::Internal(inter) => {
                            let right_map =
                                Self::restore_index_cache(inter.right(), store.clone()).unwrap();
                            cache_map.extend(right_map.iter());
                            let left_map =
                                Self::restore_index_cache(inter.left(), store.clone()).unwrap();
                            cache_map.extend(left_map.iter());
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    error!("{:?} get node error: {:?}", root_hash, e);
                }
                _ => {}
            }
        } else {
            warn!("{:?} root hash is placeholder!", root_hash);
        }
        Ok(cache_map)
    }

    ///delete node from leaf_index
    fn delete(&mut self, leaf_index: u64) -> Result<()> {
        ensure!(
            leaf_index < self.num_leaves as u64,
            "invalid leaf_index {}, num_leaves {}",
            leaf_index,
            self.num_leaves
        );
        let new_num_leaves = NodeIndex::leaves_count_end_from_index(leaf_index);
        //find deleting node by leaf_index
        let little_index = FrozenSubTreeIterator::new(new_num_leaves).collect::<Vec<_>>();
        //merge update node and index
        let update_nodes = self
            .get_all_update_nodes_from_index(leaf_index, little_index)
            .unwrap();

        let node_index = NodeIndex::new(leaf_index);
        //delete node and index
        let vec_update_nodes = update_nodes
            .values()
            .map(|v| v.clone())
            .collect::<Vec<HashValue>>();
        let vec_update_nodes_index = update_nodes
            .keys()
            .map(|v| v.clone())
            .collect::<Vec<NodeIndex>>();
        self.node_store.delete_nodes(vec_update_nodes.clone())?;
        self.node_store
            .delete_nodes_index(self.get_node_store_index_vec(vec_update_nodes_index.clone()))?;

        // update self frozen_subtree_roots
        let mut frozen_subtree_roots = self.frozen_subtree_roots.borrow_mut().to_vec();
        for hash in vec_update_nodes.clone() {
            let pos = frozen_subtree_roots
                .iter()
                .position(|x| *x == hash)
                .unwrap();
            frozen_subtree_roots.remove(pos);
        }
        self.frozen_subtree_roots = RefCell::from(frozen_subtree_roots);

        // update index cache
        for index in vec_update_nodes_index.clone() {
            self.index_cache.borrow_mut().remove(&index);
        }
        //update node number
        if node_index.is_left_child() {
            self.num_nodes = leaf_index - 1;
        } else {
            self.num_nodes = node_index.sibling().to_inorder_index();
        }
        //update self leaves number
        self.num_leaves = new_num_leaves;

        //update root hash
        let new_root_index = NodeIndex::root_from_leaf_count(self.num_leaves);
        if node_index.is_left_child() {
            //if index is left, update root hash
            self.root_hash = *self.index_cache.borrow().get(&new_root_index).unwrap();
        } else {
            self.root_hash = self
                .get_new_root_and_update_node(node_index, new_root_index)
                .unwrap();
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

    ///get all nodes larger than index of leaf_node and little update nodes
    fn get_all_update_nodes_from_index(
        &self,
        leaf_index: u64,
        little_index: Vec<NodeIndex>,
    ) -> Result<HashMap<NodeIndex, HashValue>> {
        let mut node_map = HashMap::new();
        //find larger nodes
        for index in leaf_index..self.num_nodes {
            let node_index = NodeIndex::new(index);
            match self.index_cache.borrow().get(&node_index) {
                Some(node) => {
                    node_map.insert(node_index, *node);
                }
                _ => {
                    error!(
                        "get larger nodes from leaf index: {:?}, node:{:?}",
                        leaf_index, node_index
                    );
                }
            }
        }
        //find little nodes
        for index in little_index {
            let parent_index = index.parent();
            match self.index_cache.borrow().get(&parent_index) {
                Some(node) => {
                    node_map.insert(parent_index, *node);
                }
                _ => {
                    bail!(
                        "get little nodes from index: {:?}, parent: {:?}",
                        index,
                        parent_index
                    );
                }
            }
        }
        Ok(node_map)
    }

    /// save frozen nodes
    fn save_frozen_nodes(&self, frozen_nodes: Vec<AccumulatorNode>) -> Result<()> {
        ensure!(frozen_nodes.len() > 0, "invalid frozen nodes length");
        for node in frozen_nodes {
            println!(
                "id:{:?}, save: {:?}, hash: {:?}",
                self.id.short_str(),
                node.index(),
                node.hash().short_str()
            );
            self.save_index_and_node(node.index(), node.hash(), node)?;
        }
        Ok(())
    }

    /// save node index and node object
    fn save_index_and_node(
        &self,
        _index: NodeIndex,
        _node_hash: HashValue,
        node: AccumulatorNode,
    ) -> Result<()> {
        // self.node_store
        //     .save(NodeStoreIndex::new(self.id, index), node_hash)?;
        self.node_store.save_node(node)
    }

    ///get new root by leaf index and update
    fn get_new_root_and_update_node(
        &self,
        leaf_index: NodeIndex,
        root_index: NodeIndex,
    ) -> Result<HashValue> {
        let mut right_hash = *ACCUMULATOR_PLACEHOLDER_HASH;
        let mut right_index = leaf_index.clone();
        #[allow(unused_assignments)]
        let mut new_root = right_hash;
        loop {
            //get sibling
            let sibling_index = right_index.sibling();
            if sibling_index.to_inorder_index() > leaf_index.to_inorder_index() {
                //right left replace node
                let left_hash = right_hash;
                right_hash = *ACCUMULATOR_PLACEHOLDER_HASH;
                let parent_index = right_index.parent();
                //set new root hash to parent node hash
                let parent_node =
                    AccumulatorNode::new_internal(parent_index, left_hash, right_hash);
                new_root = parent_node.hash();
                self.update_node(parent_index, new_root, parent_node.clone())?;
                if parent_index == root_index {
                    //get root node
                    break;
                }
                //for next loop
                right_index = parent_node.index();
                right_hash = new_root;
            } else {
                let sibling_hash = self.get_index(sibling_index).unwrap();
                match self.node_store.get_node(sibling_hash) {
                    Ok(Some(node)) => {
                        let left_hash = node.hash();
                        let parent_index = right_index.parent();
                        //set new root hash to parent node hash
                        let parent_node =
                            AccumulatorNode::new_internal(parent_index, left_hash, right_hash);
                        new_root = parent_node.hash();
                        self.update_node(parent_index, new_root, parent_node.clone())?;
                        if parent_index == root_index {
                            //get root node
                            break;
                        }
                        //for next loop
                        right_index = parent_node.index();
                        right_hash = new_root;
                    }
                    _ => {
                        warn!("get leaf node error: {:?}", sibling_index);
                    }
                }
            }
        }
        Ok(new_root)
    }
    /// Update node storage,and index cache
    fn update_node(&self, index: NodeIndex, hash: HashValue, node: AccumulatorNode) -> Result<()> {
        self.node_store.save_node(node.clone())?;
        self.node_store
            .save(NodeStoreIndex::new(self.id, index), hash)?;
        self.index_cache.borrow_mut().insert(index, hash);
        Ok(())
    }

    fn rightmost_leaf_index(&self) -> u64 {
        (self.num_leaves - 1) as u64
    }

    fn get_node_hash(&self, node_index: NodeIndex) -> Result<HashValue> {
        let idx = self.rightmost_leaf_index();
        if node_index.is_placeholder(idx) {
            Ok(*ACCUMULATOR_PLACEHOLDER_HASH)
        } else if node_index.is_freezable(idx) {
            // first read from cache
            Ok(self.get_index(node_index).unwrap())
        } else {
            // non-frozen non-placeholder node
            Ok(AccumulatorNode::new_internal(
                node_index,
                self.get_node_hash(node_index.left_child())?,
                self.get_node_hash(node_index.right_child())?,
            )
            .hash())
        }
    }

    /// upper bound of num of frozen nodes:
    ///     new leaves and resulting frozen internal nodes forming a complete binary subtree
    ///         num_new_leaves * 2 - 1 < num_new_leaves * 2
    ///     and the full route from root of that subtree to the accumulator root turns frozen
    ///         height - (log2(num_new_leaves) + 1) < height - 1 = root_level
    fn max_to_freeze(num_new_leaves: usize, root_level: u32) -> usize {
        precondition!(root_level as usize <= MAX_ACCUMULATOR_PROOF_DEPTH);
        precondition!(num_new_leaves < (usize::max_value() / 2));
        precondition!(num_new_leaves * 2 <= usize::max_value() - root_level as usize);
        num_new_leaves * 2 + root_level as usize
    }

    fn get_vec_hash(node_vec: Vec<AccumulatorNode>) -> Result<Vec<HashValue>> {
        let mut hash_vec = vec![];
        for node in node_vec {
            hash_vec.push(node.hash());
        }
        Ok(hash_vec)
    }
}

impl MerkleAccumulator {
    pub fn new(
        accumulator_id: HashValue,
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: LeafCount,
        num_notes: NodeCount,
        node_store: Arc<dyn AccumulatorTreeStore>,
    ) -> Result<Self> {
        let (root_hash, internal_vec) =
            Self::compute_root_hash(&frozen_subtree_roots, num_leaves).unwrap();

        Ok(Self {
            cache: Mutex::new(AccumulatorCache::new(
                accumulator_id,
                frozen_subtree_roots,
                num_leaves,
                num_notes,
                root_hash,
                internal_vec,
                node_store.clone(),
            )),
            node_store: node_store.clone(),
        })
    }

    /// Appends one leaf. This will update `frozen_subtree_roots` to store new frozen root nodes
    /// and remove old nodes if they are now part of a larger frozen subtree.
    fn _append_one(
        &self,
        frozen_subtree_roots: &mut Vec<HashValue>,
        num_existing_leaves: LeafCount,
        num_nodes: u64,
        leaf: HashValue,
    ) -> u64 {
        // First just append the leaf.
        frozen_subtree_roots.push(leaf);
        // Next, merge the last two subtrees into one. If `num_existing_leaves` has N trailing
        // ones, the carry will happen N times.
        let num_trailing_ones = (!num_existing_leaves).trailing_zeros();
        let mut num_internal_nodes = 0u64;
        let cache = self.cache.lock().unwrap();
        for i in 0..num_trailing_ones {
            let right_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let left_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
            let parent_index = NodeIndex::new(num_nodes + i as u64);
            let parent_node = AccumulatorNode::new_internal(parent_index, left_hash, right_hash);
            frozen_subtree_roots.push(parent_node.hash());
            cache
                .save_index_and_node(parent_index, parent_node.hash(), parent_node)
                .unwrap();
            num_internal_nodes += 1;
        }
        //save current leaf node
        let leaf_index = NodeIndex::new(num_nodes + num_trailing_ones as u64);
        let leaf_node = AccumulatorNode::new_leaf(leaf_index, leaf);
        cache
            .save_index_and_node(leaf_index, leaf, leaf_node)
            .unwrap();

        num_internal_nodes
    }

    /// Computes the root hash of an accumulator given the frozen subtree roots and the number of
    /// leaves in this accumulator.
    fn compute_root_hash(
        frozen_subtree_roots: &[HashValue],
        num_leaves: LeafCount,
    ) -> Result<(HashValue, Vec<HashValue>)> {
        let mut hash_vec = vec![];
        match frozen_subtree_roots.len() {
            0 => return Ok((*ACCUMULATOR_PLACEHOLDER_HASH, hash_vec)),
            1 => {
                hash_vec.push(frozen_subtree_roots[0]);
                return Ok((frozen_subtree_roots[0], hash_vec));
            }
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
            hash_vec.push(current_hash);
            println!("insert :{:?}", current_hash.short_str());
            bitmap >>= 1;
            index += 1;
        }

        Ok((current_hash, hash_vec))
    }
}

impl Accumulator for MerkleAccumulator {
    fn append(&self, new_leaves: &[HashValue]) -> Result<(HashValue, u64), Error> {
        let mut cache_guard = self.cache.lock().unwrap();
        let cache = cache_guard.deref_mut();
        let first_index_leaf = cache.num_leaves;
        let (root_hash, frozen_nodes) = cache.append_leaves(new_leaves).unwrap();
        cache.save_frozen_nodes(frozen_nodes)?;
        Ok((root_hash, first_index_leaf))
    }

    fn append_only_cache(&self, leaves: &[HashValue]) -> Result<(HashValue, u64), Error> {
        let mut cache_guard = self.cache.lock().unwrap();
        let cache = cache_guard.deref_mut();
        let first_index_leaf = cache.num_leaves;
        let (root_hash, _frozen_nodes) = cache.append_leaves(leaves).unwrap();
        Ok((root_hash, first_index_leaf))
    }

    fn get_leaf(&self, leaf_index: u64) -> Result<Option<HashValue>, Error> {
        Ok(Some(
            self.cache
                .lock()
                .unwrap()
                .get_node_hash(NodeIndex::new(leaf_index))
                .unwrap(),
        ))
    }

    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>, Error> {
        let cache = self.cache.lock().unwrap();
        ensure!(
            leaf_index < cache.num_leaves as u64,
            "get proof invalid leaf_index {}, num_leaves {}",
            leaf_index,
            cache.num_leaves
        );

        let siblings = cache.get_siblings(leaf_index, |_p| true)?;
        Ok(Some(AccumulatorProof::new(siblings)))
    }

    fn root_hash(&self) -> HashValue {
        self.cache.lock().unwrap().root_hash
    }

    fn num_leaves(&self) -> u64 {
        self.cache.lock().unwrap().num_leaves
    }

    fn num_nodes(&self) -> u64 {
        self.cache.lock().unwrap().num_nodes
    }

    fn update(&self, leaf_index: u64, leaves: &[HashValue]) -> Result<(HashValue, u64), Error> {
        let mut cache_guard = self.cache.lock().unwrap();
        let cache = cache_guard.deref_mut();
        //ensure leaves is null
        ensure!(leaves.len() > 0, "invalid leaves len: {}", leaves.len());
        ensure!(
            leaf_index < cache.num_leaves as u64,
            "update invalid leaf_index {}, num_leaves {}",
            leaf_index,
            cache.num_leaves
        );
        // delete larger nodes from index
        cache.delete(leaf_index)?;
        // append new notes
        let (root, _) = cache.append_leaves(leaves).unwrap();
        Ok((root, leaf_index))
    }

    fn get_frozen_subtree_roots(&self) -> Result<Vec<HashValue>, Error> {
        let cache = self.cache.lock().unwrap();
        let result = FrozenSubTreeIterator::new(cache.num_leaves)
            .map(|p| cache.get_index(p).unwrap())
            .collect::<Vec<_>>();

        Ok(result)
    }
}

pub struct MockAccumulatorStore {
    index_store: RefCell<HashMap<NodeStoreIndex, HashValue>>,
    node_store: RefCell<HashMap<HashValue, AccumulatorNode>>,
}

impl MockAccumulatorStore {
    pub fn new() -> Self {
        Self {
            index_store: RefCell::new(HashMap::new()),
            node_store: RefCell::new(HashMap::new()),
        }
    }
}

impl AccumulatorTreeStore for MockAccumulatorStore {}
impl AccumulatorReader for MockAccumulatorStore {
    fn get(&self, index: NodeStoreIndex) -> Result<Option<AccumulatorNode>, Error> {
        match self.index_store.borrow().get(&index) {
            Some(node_index) => match self.node_store.borrow().get(node_index) {
                Some(node) => Ok(Some(node.clone())),
                None => bail!("get node is null:{:?}", index),
            },
            None => bail!("get node index is null:{:?}", index),
        }
    }

    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        match self.node_store.borrow().get(&hash) {
            Some(node) => Ok(Some(node.clone())),
            None => bail!("get node is null: {}", hash),
        }
    }
}
impl AccumulatorWriter for MockAccumulatorStore {
    fn save(&self, index: NodeStoreIndex, hash: HashValue) -> Result<(), Error> {
        self.index_store.borrow_mut().insert(index, hash);
        Ok(())
    }

    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.node_store.borrow_mut().insert(node.hash(), node);
        Ok(())
    }

    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<(), Error> {
        for hash in node_hash_vec {
            self.node_store.borrow_mut().remove(&hash);
        }
        Ok(())
    }

    fn delete_nodes_index(&self, index_vec: Vec<NodeStoreIndex>) -> Result<(), Error> {
        ensure!(
            index_vec.len() > 0,
            " invalid index vec len: {}.",
            index_vec.len()
        );
        for index in index_vec {
            self.index_store.borrow_mut().remove(&index);
        }
        Ok(())
    }
}
