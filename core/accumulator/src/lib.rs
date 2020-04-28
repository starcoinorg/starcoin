// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::node_index::NodeIndex;
use crate::proof::AccumulatorProof;
use crate::tree::AccumulatorTree;
use crate::tree_store::AccumulatorCache;
use anyhow::{ensure, Error, Result};
use logger::prelude::*;
pub use node::AccumulatorNode;
use parking_lot::Mutex;
use starcoin_crypto::HashValue;
use std::sync::Arc;

#[cfg(test)]
mod accumulator_test;
pub mod node;
pub mod node_index;
mod proof;
mod tree;
mod tree_store;

pub type LeafCount = u64;
pub type NodeCount = u64;

pub const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;
pub const MAX_ACCUMULATOR_LEAVES: LeafCount = 1 << MAX_ACCUMULATOR_PROOF_DEPTH;
pub const MAC_CACHE_SIZE: usize = 65535;

/// accumulator method define
pub trait Accumulator {
    /// Append leaves and return new root
    fn append(&self, leaves: &[HashValue]) -> Result<(HashValue, u64), Error>;
    /// Get leaf node by index.
    fn get_leaf(&self, leaf_index: u64) -> Result<Option<HashValue>, Error>;
    /// Get proof by leaf index.
    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>>;
    /// Get accumulator node by hash.
    fn get_node(&self, hash: HashValue) -> Result<AccumulatorNode>;
    /// Flush node to storage.
    fn flush(&self) -> Result<()>;
    /// Get current accumulator tree root hash.
    fn root_hash(&self) -> HashValue;
    /// Get current accumulator tree number of leaves.
    fn num_leaves(&self) -> u64;
    /// Get current accumulator tree number of nodes.
    fn num_nodes(&self) -> u64;
    /// Get frozen subtree roots.
    fn get_frozen_subtree_roots(&self) -> Result<Vec<HashValue>>;
}

pub trait AccumulatorReader {
    ///get node by node hash
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>>;
    /// multiple get nodes
    fn multiple_get(&self, hash_vec: Vec<HashValue>) -> Result<Vec<AccumulatorNode>>;
}

pub trait AccumulatorWriter {
    /// save node
    fn save_node(&self, node: AccumulatorNode) -> Result<()>;
    /// batch save nodes
    fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> Result<()>;
    ///delete node
    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<()>;
}

pub trait AccumulatorTreeStore:
    AccumulatorReader + AccumulatorWriter + std::marker::Send + std::marker::Sync
{
}

/// MerkleAccumulator is a accumulator algorithm implement and it is stateless.
pub struct MerkleAccumulator {
    update_nodes: Mutex<Vec<AccumulatorNode>>,
    tree: Mutex<AccumulatorTree>,
    node_store: Arc<dyn AccumulatorTreeStore>,
}

impl MerkleAccumulator {
    pub fn new(
        accumulator_id: HashValue,
        root_hash: HashValue,
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: LeafCount,
        num_notes: NodeCount,
        node_store: Arc<dyn AccumulatorTreeStore>,
    ) -> Result<Self> {
        Ok(Self {
            update_nodes: Mutex::new(vec![]),
            tree: Mutex::new(AccumulatorTree::new(
                accumulator_id,
                frozen_subtree_roots,
                num_leaves,
                num_notes,
                root_hash,
                node_store.clone(),
            )),
            node_store: node_store.clone(),
        })
    }
    #[cfg(test)]
    pub fn get_node_from_cache(&self, hash: HashValue) -> AccumulatorNode {
        AccumulatorCache::get_node(hash)
    }
    #[cfg(test)]
    pub fn get_node_from_storage(&self, hash: HashValue) -> AccumulatorNode {
        self.node_store.get_node(hash).unwrap().unwrap()
    }
}

impl Accumulator for MerkleAccumulator {
    fn append(&self, new_leaves: &[HashValue]) -> Result<(HashValue, u64), Error> {
        let mut tree_guard = self.tree.lock();
        let first_index_leaf = tree_guard.num_leaves;
        let (root_hash, frozen_nodes) = tree_guard.append_leaves(new_leaves).unwrap();
        self.update_nodes.lock().extend_from_slice(&frozen_nodes);
        Ok((root_hash, first_index_leaf))
    }

    fn get_leaf(&self, leaf_index: u64) -> Result<Option<HashValue>, Error> {
        Ok(Some(
            self.tree
                .lock()
                .get_node_hash(NodeIndex::new(leaf_index))
                .unwrap(),
        ))
    }

    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>, Error> {
        let tree_guard = self.tree.lock();
        ensure!(
            leaf_index < tree_guard.num_leaves as u64,
            "get proof invalid leaf_index {}, num_leaves {}",
            leaf_index,
            tree_guard.num_leaves
        );

        let siblings = tree_guard.get_siblings(leaf_index, |_p| true)?;
        Ok(Some(AccumulatorProof::new(siblings)))
    }

    fn get_node(&self, hash: HashValue) -> Result<AccumulatorNode, Error> {
        self.tree.lock().get_node(hash)
    }

    fn flush(&self) -> Result<(), Error> {
        let mut nodes = self.update_nodes.lock();
        if !nodes.is_empty() {
            self.node_store.save_nodes(nodes.to_vec())?;
            nodes.clear();
            info!("flush node to storage ok!");
        }
        Ok(())
    }

    fn root_hash(&self) -> HashValue {
        self.tree.lock().root_hash
    }
    fn num_leaves(&self) -> u64 {
        self.tree.lock().num_leaves
    }
    fn num_nodes(&self) -> u64 {
        self.tree.lock().num_nodes
    }

    fn get_frozen_subtree_roots(&self) -> Result<Vec<HashValue>, Error> {
        self.tree.lock().get_frozen_subtree_roots()
    }
}
