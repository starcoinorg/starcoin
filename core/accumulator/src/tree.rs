// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0s

use crate::node::ACCUMULATOR_PLACEHOLDER_HASH;
use crate::node_index::MAX_ACCUMULATOR_PROOF_DEPTH;
use crate::node_index::NODE_ERROR_INDEX;
use crate::{
    AccumulatorNode, AccumulatorTreeStore, LeafCount, NodeCount, GLOBAL_NODE_PARENT_CACHE,
};
use anyhow::{ensure, Result};
use logger::prelude::*;
use mirai_annotations::*;
use starcoin_crypto::HashValue;
use std::sync::Arc;

pub struct AccumulatorTree {
    /// Accumulator id
    id: HashValue,
    /// forzen subtree roots hashes.
    frozen_subtree_roots: Vec<HashValue>,
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
        trace!("accumulator cache new: {:?}", accumulator_id.short_str());
        Self {
            id: accumulator_id,
            frozen_subtree_roots,
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

        let mut node_vec = vec![];
        let mut frozen_subtree_roots = self.frozen_subtree_roots.clone();
        let mut num_leaves = self.num_leaves;
        let mut num_nodes = self.num_nodes;
        for leaf in new_leaves {
            // First just append the leaf.
            node_vec.push(AccumulatorNode::new_leaf(
                NODE_ERROR_INDEX.to_owned(),
                *leaf,
            ));
            num_nodes += 1;
            frozen_subtree_roots.push(*leaf);
            let num_trailing_ones = (!num_leaves).trailing_zeros();
            for _i in 0..num_trailing_ones {
                let right_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
                let left_hash = frozen_subtree_roots.pop().expect("Invalid accumulator.");
                let parent_node = AccumulatorNode::new_internal(
                    NODE_ERROR_INDEX.to_owned(),
                    left_hash,
                    right_hash,
                );
                node_vec.push(parent_node.clone());
                num_nodes += 1;
                frozen_subtree_roots.push(parent_node.hash());
            }
            num_leaves += 1;
        }

        ensure!(
            frozen_subtree_roots.len() == num_leaves.count_ones() as usize,
            "The number of frozen subtrees does not match the number of leaves. \
             frozen_subtree_roots.len(): {}. num_leaves: {}.",
            frozen_subtree_roots.len(),
            num_leaves,
        );

        let root_hash = Self::compute_root_hash(&frozen_subtree_roots, num_leaves);

        self.root_hash = root_hash;
        //TODO frozen_subtree must recomputer
        self.frozen_subtree_roots = frozen_subtree_roots.clone();
        self.num_leaves = num_leaves;
        self.num_nodes = num_nodes;

        Ok((root_hash, node_vec))
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

    fn save_parent_cache(self, parent: HashValue, left: HashValue, right: HashValue) -> Result<()> {
        let mut cache = GLOBAL_NODE_PARENT_CACHE.lock();
        if left != *ACCUMULATOR_PLACEHOLDER_HASH {
            cache.put(left, parent);
        }
        if right != *ACCUMULATOR_PLACEHOLDER_HASH {
            cache.put(right, parent);
        }
        Ok(())
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

        while bitmap > 0 {
            current_hash = if bitmap & 1 != 0 {
                AccumulatorNode::new_internal(
                    NODE_ERROR_INDEX.to_owned(),
                    *frozen_subtree_iter
                        .next()
                        .expect("This frozen subtree should exist."),
                    current_hash,
                )
            } else {
                AccumulatorNode::new_internal(
                    NODE_ERROR_INDEX.to_owned(),
                    current_hash,
                    *ACCUMULATOR_PLACEHOLDER_HASH,
                )
            }
            .hash();
            bitmap >>= 1;
        }

        current_hash
    }

    pub(crate) fn get_frozen_subtree_roots(&self) -> Result<Vec<HashValue>> {
        Ok(self.frozen_subtree_roots.clone())
    }

    /// filter function can be applied to filter out certain siblings.
    pub(crate) fn get_siblings(&self, leaf_hash: HashValue) -> Result<Vec<HashValue>> {
        let mut temp_node_hash = leaf_hash;
        let root_hash = self.root_hash;
        let mut siblings = vec![];
        let mut cache = GLOBAL_NODE_PARENT_CACHE.lock();
        loop {
            let parent_node_hash = cache.get(&temp_node_hash);
            match parent_node_hash {
                Some(node) => {
                    if *node == root_hash {
                        break;
                    }
                    let node = self.get_node(*node);
                    if !node.is_empty() {
                        match node {
                            AccumulatorNode::Internal(internal) => {
                                if temp_node_hash == internal.left() {
                                    siblings.push(internal.right());
                                } else {
                                    siblings.push(internal.left());
                                }
                                temp_node_hash = internal.hash();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {} //TODO get storage reduction parent mapping cache
            }
        }

        Ok(siblings)
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
        (self.num_leaves - 1) as u64
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
