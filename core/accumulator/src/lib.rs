// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Error, Result};

use starcoin_crypto::HashValue;

use crate::proof::AccumulatorProof;
use crate::tree::AccumulatorTree;
use lru::LruCache;
pub use node::AccumulatorNode;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
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
/// Global node cache
pub static GLOBAL_NODE_CACHE: Lazy<Mutex<LruCache<HashValue, AccumulatorNode>>> =
    Lazy::new(|| Mutex::new(LruCache::new(MAC_CACHE_SIZE)));
/// Global node parent mapping cache.
pub static GLOBAL_NODE_PARENT_CACHE: Lazy<Mutex<LruCache<HashValue, HashValue>>> =
    Lazy::new(|| Mutex::new(LruCache::new(MAC_CACHE_SIZE)));

/// accumulator method define
pub trait Accumulator {
    /// Append leaves and return new root
    fn append(&self, leaves: &[HashValue]) -> Result<(HashValue, u64), Error>;
    /// Get proof by leaf hash.
    fn get_proof(&self, leaf_hash: HashValue) -> Result<Option<AccumulatorProof>>;
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
    ///delete node
    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<()>;
}

pub trait AccumulatorTreeStore:
    AccumulatorReader + AccumulatorWriter + std::marker::Send + std::marker::Sync
{
}

/// MerkleAccumulator is a accumulator algorithm implement and it is stateless.
pub struct MerkleAccumulator {
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
}

impl Accumulator for MerkleAccumulator {
    fn append(&self, new_leaves: &[HashValue]) -> Result<(HashValue, u64), Error> {
        let mut tree_guard = self.tree.lock();
        let first_index_leaf = tree_guard.num_leaves;
        let (root_hash, _frozen_nodes) = tree_guard.append_leaves(new_leaves).unwrap();
        Ok((root_hash, first_index_leaf))
    }

    fn get_proof(&self, leaf_hash: HashValue) -> Result<Option<AccumulatorProof>, Error> {
        let tree_guard = self.tree.lock();
        match tree_guard.get_siblings(leaf_hash) {
            Ok(siblings) => Ok(Some(AccumulatorProof::new(siblings))),
            _ => bail!("get proof error:{:?}", leaf_hash),
        }
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
