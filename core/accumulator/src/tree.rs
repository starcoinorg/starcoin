// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0s

use crate::node::ACCUMULATOR_PLACEHOLDER_HASH;
use crate::node_index::FrozenSubTreeIterator;
use crate::node_index::{NodeIndex, MAX_ACCUMULATOR_PROOF_DEPTH};
use crate::tree_store::NodeCacheKey;
use crate::{AccumulatorNode, AccumulatorTreeStore, LeafCount, NodeCount, MAC_CACHE_SIZE};
use anyhow::Result;
use logger::prelude::*;
use lru::LruCache;
use mirai_annotations::*;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
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
    /// The index cache
    index_cache: Lazy<Mutex<LruCache<NodeCacheKey, HashValue>>>,
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
            index_cache: Lazy::new(|| Mutex::new(LruCache::new(MAC_CACHE_SIZE))),
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

            new_num_nodes += 1;
            let mut pos = leaf_pos;
            while pos.is_right_child() {
                // let mut internal_node = AccumulatorNode::Empty;
                let sibling = pos.sibling();

                let internal_node = match left_siblings.pop() {
                    Some((x, left_hash)) => {
                        assert_eq!(x, sibling);
                        AccumulatorNode::new_internal(pos.parent(), left_hash, hash)
                    }
                    None => AccumulatorNode::new_internal(
                        pos.parent(),
                        self.get_node_hash(sibling).unwrap(),
                        hash,
                    ),
                };
                hash = internal_node.hash();
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
        self.num_leaves = last_new_leaf_count;
        self.frozen_subtree_roots = FrozenSubTreeIterator::new(last_new_leaf_count)
            .map(|p| self.get_node_hash(p).unwrap())
            .collect::<Vec<_>>();
        self.num_nodes = new_num_nodes;

        Ok((hash, not_frozen_nodes))
    }

    /// Get node for self package.
    pub(crate) fn get_node(&self, hash: HashValue) -> Result<AccumulatorNode> {
        let node = match self.store.clone().get_node(hash) {
            Ok(Some(node)) => node,
            _ => {
                error!("get accumulator node from store err:{:?}", hash.short_str());
                AccumulatorNode::new_empty()
            }
        };
        Ok(node)
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
            let node_hash = self.get_node_hash_always(node_index);
            Ok(node_hash)
        }
    }

    /// Update node to cache.
    fn update_cache(&self, node_vec: Vec<AccumulatorNode>) -> Result<()> {
        trace!("accumulator update cache.");
        self.save_node_indexes(node_vec)
    }

    fn get_node_index(&self, key: NodeCacheKey) -> HashValue {
        match self.index_cache.lock().get(&key) {
            Some(node_hash) => *node_hash,
            None => {
                warn!("get node index hash error:{:?}", key);
                HashValue::zero()
            }
        }
    }

    /// Get node hash always.
    fn get_node_hash_always(&self, index: NodeIndex) -> HashValue {
        // get hash from cache
        let mut temp_index = index;
        let mut index_key = NodeCacheKey::new(self.id, temp_index);
        let node_hash = self.get_node_index(index_key);
        if node_hash != HashValue::zero() {
            return node_hash;
        }
        // find parent hash,then get node by parent hash
        let root_index = NodeIndex::root_from_leaf_count(self.num_leaves);
        let level = root_index.level();
        let mut parent_hash = node_hash;
        for _i in 0..level {
            index_key = NodeCacheKey::new(self.id, temp_index.parent());
            parent_hash = self.get_node_index(index_key);
            if parent_hash != HashValue::zero() {
                break;
            }
            temp_index = temp_index.parent();
        }

        if parent_hash == HashValue::zero() {
            error!("get node index parent hash is zero!");
            parent_hash = self.root_hash;
        }

        // get node by hash
        let mut hash_vec = vec![parent_hash];
        for _i in 0..level {
            while let Some(temp_node_hash) = hash_vec.pop() {
                match self.get_node(temp_node_hash) {
                    Ok(AccumulatorNode::Internal(internal)) => {
                        let internal_index = internal.index();
                        if internal_index == index.parent() {
                            if internal_index.left_child() == index {
                                return internal.left();
                            }
                            if internal_index.right_child() == index {
                                return internal.right();
                            }
                        } else {
                            hash_vec.push(internal.left());
                            hash_vec.push(internal.right());
                        }
                    }
                    Ok(AccumulatorNode::Leaf(leaf)) => {
                        if leaf.index() == index {
                            return leaf.value();
                        }
                    }
                    _ => {
                        // println!("get node error:{:?}", temp_node_hash);
                    }
                }
            }
        }
        node_hash
    }

    fn save_node_indexes(&self, nodes: Vec<AccumulatorNode>) -> Result<()> {
        let mut cache = self.index_cache.lock();
        for node in nodes {
            // println!("{:?}: {:?}", node.index(), node.hash().short_str());
            if let Some(old) = cache.put(NodeCacheKey::new(self.id, node.index()), node.hash()) {
                warn!(
                    "cache exist node hash: {:?}-{:?}-{:?}",
                    self.id.short_str(),
                    node.index(),
                    old
                );
            }
        }
        Ok(())
    }

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
