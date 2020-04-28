// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0s

use crate::node::ACCUMULATOR_PLACEHOLDER_HASH;
use crate::node_index::{FrozenSubTreeIterator, NODE_ERROR_INDEX};
use crate::node_index::{NodeIndex, MAX_ACCUMULATOR_PROOF_DEPTH};
use crate::tree_store::AccumulatorCache;
use crate::{AccumulatorNode, AccumulatorTreeStore, LeafCount, NodeCount};
use anyhow::{ensure, Result};
use logger::prelude::*;
use mirai_annotations::*;
use starcoin_crypto::HashValue;
use std::cell::RefCell;
use std::sync::Arc;

pub struct AccumulatorTree {
    /// Accumulator id
    id: HashValue,
    /// forzen subtree roots hashes.
    frozen_subtree_roots: RefCell<Vec<HashValue>>,
    /// The total number of leaves in this accumulator.
    pub(crate) num_leaves: LeafCount,
    /// The total number of nodes in this accumulator.
    pub(crate) num_nodes: NodeCount,
    /// The root hash of this accumulator.
    pub(crate) root_hash: HashValue,
    /// The storage of accumulator.
    store: Arc<dyn AccumulatorTreeStore>,
}

impl AccumulatorTree {
    pub fn new(
        accumulator_id: HashValue,
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: LeafCount,
        num_nodes: NodeCount,
        root_hash: HashValue,
        store: Arc<dyn AccumulatorTreeStore>,
    ) -> Self {
        info!("accumulator cache new: {:?}", accumulator_id.short_str());
        Self::restore_index_cache(accumulator_id, frozen_subtree_roots.clone(), store.clone())
            .unwrap();
        Self {
            id: accumulator_id,
            frozen_subtree_roots: RefCell::new(frozen_subtree_roots),
            num_leaves,
            num_nodes,
            root_hash,
            store,
        }
    }

    ///append from multiple leaves
    pub(crate) fn append_leaves(
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
            new_num_nodes += 1;
            let mut pos = leaf_pos;
            while pos.is_right_child() {
                let mut internal_node = AccumulatorNode::Empty;
                let sibling = pos.sibling();

                hash = match left_siblings.pop() {
                    Some((x, left_hash)) => {
                        assert_eq!(x, sibling);
                        internal_node =
                            AccumulatorNode::new_internal(pos.parent(), left_hash, hash);
                        internal_node.hash()
                    }
                    None => {
                        internal_node = AccumulatorNode::new_internal(
                            pos.parent(),
                            self.get_node_hash(sibling).unwrap(),
                            hash,
                        );
                        internal_node.hash()
                    }
                };
                pos = pos.parent();
                to_freeze.push(internal_node);
                new_num_nodes += 1;
            }
            // The node remaining must be a left child, possibly the root of a complete binary tree.
            left_siblings.push((pos, hash));
        }

        let mut not_frozen_nodes = vec![];
        // Now reconstruct the final root hash by walking up to root level and adding
        // placeholder hash nodes as needed on the right, and left siblings that have either
        // been newly created or read from storage.
        let (mut pos, mut hash) = left_siblings.pop().expect("Must have at least one node");
        for _ in pos.level()..root_level as u32 {
            hash = if pos.is_left_child() {
                let not_frozen = AccumulatorNode::new_internal(
                    pos.parent(),
                    hash,
                    *ACCUMULATOR_PLACEHOLDER_HASH,
                );
                not_frozen_nodes.push(not_frozen.clone());
                not_frozen.hash()
            } else {
                let sibling = pos.sibling();
                match left_siblings.pop() {
                    Some((x, left_hash)) => {
                        assert_eq!(x, sibling);
                        let not_frozen =
                            AccumulatorNode::new_internal(pos.parent(), left_hash, hash);
                        not_frozen_nodes.push(not_frozen.clone());
                        not_frozen.hash()
                    }
                    None => {
                        let not_frozen = AccumulatorNode::new_internal(
                            pos.parent(),
                            self.get_node_hash(sibling).unwrap(),
                            hash,
                        );
                        not_frozen_nodes.push(not_frozen.clone());
                        not_frozen.hash()
                    }
                }
            };
            pos = pos.parent();
        }
        assert!(left_siblings.is_empty());
        //update frozen tag
        to_freeze = to_freeze
            .iter()
            .map(|node| {
                node.clone().frozen().unwrap();
                node.clone()
            })
            .collect();
        //aggregator all nodes
        not_frozen_nodes.extend_from_slice(&to_freeze);
        // udpate to cache
        self.update_cache(not_frozen_nodes.clone())?;
        // update self properties
        self.root_hash = hash;
        self.frozen_subtree_roots = RefCell::new(
            FrozenSubTreeIterator::new(last_new_leaf_count)
                .map(|p| self.get_node_hash(p).unwrap())
                .collect::<Vec<_>>(),
        );
        self.num_leaves = last_new_leaf_count;
        self.num_nodes = new_num_nodes;

        Ok((hash, not_frozen_nodes))
    }

    /// Get accumulator node by hash.
    fn get_node(&self, node_hash: HashValue) -> AccumulatorNode {
        match self.store.clone().get_node(node_hash) {
            Ok(Some(node)) => node,
            _ => {
                error!("get accumulator node err:{:?}", node_hash);
                AccumulatorNode::new_empty()
            }
        }
    }
    /// Get accumulator node by hash, first from cache ,if not exist,then through store.
    fn get_node_through_cache(
        hash: HashValue,
        store: Arc<dyn AccumulatorTreeStore>,
    ) -> AccumulatorNode {
        let mut node = AccumulatorCache::get_node(hash);
        if node.is_empty() {
            node = match store.clone().get_node(hash) {
                Ok(Some(node)) => node,
                _ => {
                    error!("get accumulator node from store err:{:?}", hash);
                    AccumulatorNode::new_empty()
                }
            }
        }
        node
    }

    pub(crate) fn get_frozen_subtree_roots(&self) -> Result<Vec<HashValue>> {
        let result = FrozenSubTreeIterator::new(self.num_leaves)
            .map(|p| self.get_node_hash(p).unwrap())
            .collect::<Vec<_>>();
        Ok(result)
    }

    /// filter function can be applied to filter out certain siblings.
    pub(crate) fn get_siblings(
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

    /// Get node hash by index.
    pub(crate) fn get_node_hash(&self, node_index: NodeIndex) -> Result<HashValue> {
        let idx = self.rightmost_leaf_index();
        if node_index.is_placeholder(idx) {
            Ok(*ACCUMULATOR_PLACEHOLDER_HASH)
        } else {
            let node_hash = AccumulatorCache::get_node_hash(self.id, node_index);
            if node_hash == HashValue::zero() {
                // get from to storage
                let parent = node_index.parent();
                let parent_hash = AccumulatorCache::get_node_hash(self.id, node_index);
                if parent_hash != HashValue::zero() {
                } else {
                    error!("get parent node null: {:?}", parent);
                    // Ok(HashValue::zero())
                }
            }
            Ok(node_hash)
        }
    }

    pub(crate) fn restore_index_cache(
        accumulator_id: HashValue,
        hashes: Vec<HashValue>,
        store: Arc<dyn AccumulatorTreeStore>,
    ) -> Result<()> {
        for hash in hashes {
            let node = AccumulatorTree::get_node_through_cache(hash, store.clone());
            match node {
                AccumulatorNode::Internal(internal) => {
                    AccumulatorCache::save_node_index(accumulator_id, internal.index(), hash)?;
                    let mut two_hash = vec![];
                    two_hash.push(internal.left());
                    two_hash.push(internal.right());
                    AccumulatorTree::restore_index_cache(
                        accumulator_id,
                        two_hash.clone(),
                        store.clone(),
                    )?;
                }
                AccumulatorNode::Leaf(leaf) => {
                    AccumulatorCache::save_node_index(accumulator_id, leaf.index(), hash)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// update node to cache
    fn update_cache(&self, node_vec: Vec<AccumulatorNode>) -> Result<()> {
        info!("accumulator update cache.");
        AccumulatorCache::save_nodes(node_vec.clone())?;
        AccumulatorCache::save_node_indexes(self.id, node_vec)
    }

    // ///get new root by leaf index and update
    // fn get_new_root_and_update_node(
    //     &self,
    //     leaf_index: NodeIndex,
    //     root_index: NodeIndex,
    // ) -> Result<HashValue> {
    //     let mut right_hash = *ACCUMULATOR_PLACEHOLDER_HASH;
    //     let mut right_index = leaf_index.clone();
    //     #[allow(unused_assignments)]
    //     let mut new_root = right_hash;
    //     loop {
    //         //get sibling
    //         let sibling_index = right_index.sibling();
    //         if sibling_index.to_inorder_index() > leaf_index.to_inorder_index() {
    //             //right left replace node
    //             let left_hash = right_hash;
    //             right_hash = *ACCUMULATOR_PLACEHOLDER_HASH;
    //             let parent_index = right_index.parent();
    //             //set new root hash to parent node hash
    //             let parent_node =
    //                 AccumulatorNode::new_internal(parent_index, left_hash, right_hash);
    //             new_root = parent_node.hash();
    //             self.update_node(parent_index, new_root, parent_node.clone())?;
    //             if parent_index == root_index {
    //                 //get root node
    //                 break;
    //             }
    //             //for next loop
    //             right_index = parent_node.index();
    //             right_hash = new_root;
    //         } else {
    //             let sibling_hash = self.get_index(sibling_index).unwrap();
    //             match self.node_store.get_node(sibling_hash) {
    //                 Ok(Some(node)) => {
    //                     let left_hash = node.hash();
    //                     let parent_index = right_index.parent();
    //                     //set new root hash to parent node hash
    //                     let parent_node =
    //                         AccumulatorNode::new_internal(parent_index, left_hash, right_hash);
    //                     new_root = parent_node.hash();
    //                     self.update_node(parent_index, new_root, parent_node.clone())?;
    //                     if parent_index == root_index {
    //                         //get root node
    //                         break;
    //                     }
    //                     //for next loop
    //                     right_index = parent_node.index();
    //                     right_hash = new_root;
    //                 }
    //                 _ => {
    //                     warn!("get leaf node error: {:?}", sibling_index);
    //                 }
    //             }
    //         }
    //     }
    //     Ok(new_root)
    // }
    /// Update node storage,and index cache
    // fn update_node(&self, index: NodeIndex, hash: HashValue, node: AccumulatorNode) -> Result<()> {
    //     self.node_store.save_node(node.clone())?;
    //     self.index_cache.borrow_mut().insert(index, hash);
    //     Ok(())
    // }

    fn rightmost_leaf_index(&self) -> u64 {
        if self.num_leaves == 0 {
            0 as u64
        } else {
            (self.num_leaves - 1) as u64
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
}
