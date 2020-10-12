// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::accumulator_info::AccumulatorInfo;
use crate::node_index::NodeIndex;
use crate::proof::AccumulatorProof;
use crate::tree::AccumulatorTree;
use anyhow::{ensure, Error, Result};
pub use node::AccumulatorNode;
use parking_lot::Mutex;
use starcoin_crypto::hash::ACCUMULATOR_PLACEHOLDER_HASH;
use starcoin_crypto::HashValue;
use std::sync::Arc;
pub use tree_store::AccumulatorTreeStore;

pub mod accumulator_info;
#[cfg(test)]
mod accumulator_test;
pub mod node;
pub mod node_index;
mod proof;
mod tree;
pub mod tree_store;

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
    /// Get node by position.
    fn get_node_by_position(&self, position: u64) -> Result<Option<HashValue>>;
    /// Get proof by leaf index.
    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>>;
    /// Get accumulator node by hash.
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>>;
    /// Flush node to storage.
    fn flush(&self) -> Result<()>;
    /// Get current accumulator tree root hash.
    fn root_hash(&self) -> HashValue;
    /// Get current accumulator tree number of leaves.
    fn num_leaves(&self) -> u64;
    /// Get current accumulator tree number of nodes.
    fn num_nodes(&self) -> u64;
    /// Get frozen subtree roots.
    fn get_frozen_subtree_roots(&self) -> Vec<HashValue>;
    /// Get accumulator info
    fn get_info(&self) -> AccumulatorInfo;
}

/// MerkleAccumulator is a accumulator algorithm implement and it is stateless.
pub struct MerkleAccumulator {
    tree: Mutex<AccumulatorTree>,
}

impl MerkleAccumulator {
    pub fn new(
        root_hash: HashValue,
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: LeafCount,
        num_notes: NodeCount,
        node_store: Arc<dyn AccumulatorTreeStore>,
    ) -> Self {
        Self {
            tree: Mutex::new(AccumulatorTree::new(
                frozen_subtree_roots,
                num_leaves,
                num_notes,
                root_hash,
                node_store,
            )),
        }
    }

    pub fn new_with_info(
        acc_info: AccumulatorInfo,
        node_store: Arc<dyn AccumulatorTreeStore>,
    ) -> Self {
        Self {
            tree: Mutex::new(AccumulatorTree::new(
                acc_info.frozen_subtree_roots,
                acc_info.num_leaves,
                acc_info.num_nodes,
                acc_info.accumulator_root,
                node_store,
            )),
        }
    }

    pub fn new_empty(node_store: Arc<dyn AccumulatorTreeStore>) -> Self {
        Self {
            tree: Mutex::new(AccumulatorTree::new(
                vec![],
                0,
                0,
                *ACCUMULATOR_PLACEHOLDER_HASH,
                node_store,
            )),
        }
    }
}

impl Accumulator for MerkleAccumulator {
    fn append(&self, new_leaves: &[HashValue]) -> Result<(HashValue, u64), Error> {
        let mut tree_guard = self.tree.lock();
        let first_index_leaf = tree_guard.num_leaves;
        let root_hash = tree_guard.append_leaves(new_leaves)?;
        Ok((root_hash, first_index_leaf))
    }

    fn get_leaf(&self, leaf_index: u64) -> Result<Option<HashValue>, Error> {
        Ok(Some(
            self.tree
                .lock()
                .get_node_hash(NodeIndex::from_leaf_index(leaf_index))?,
        ))
    }

    fn get_node_by_position(&self, position: u64) -> Result<Option<HashValue>, Error> {
        Ok(Some(
            self.tree.lock().get_node_hash(NodeIndex::new(position))?,
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

    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>, Error> {
        self.tree.lock().get_node(hash)
    }

    fn flush(&self) -> Result<(), Error> {
        self.tree.lock().flush()
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

    fn get_frozen_subtree_roots(&self) -> Vec<HashValue> {
        self.tree.lock().get_frozen_subtree_roots()
    }

    fn get_info(&self) -> AccumulatorInfo {
        AccumulatorInfo::new(
            self.root_hash(),
            self.get_frozen_subtree_roots(),
            self.num_leaves(),
            self.num_nodes(),
        )
    }
}
