// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::node_index::NodeIndex;
use crate::{
    AccumulatorNode, AccumulatorReader, AccumulatorTreeStore, AccumulatorWriter, MAC_CACHE_SIZE,
};
use anyhow::{bail, Error, Result};
use logger::prelude::*;
use lru::LruCache;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use std::collections::HashMap;

/// Global node cache
pub static GLOBAL_NODE_CACHE: Lazy<Mutex<LruCache<HashValue, AccumulatorNode>>> =
    Lazy::new(|| Mutex::new(LruCache::new(MAC_CACHE_SIZE)));
/// Global node index  cache.
pub static GLOBAL_NODE_INDEX_CACHE: Lazy<Mutex<LruCache<NodeCacheKey, HashValue>>> =
    Lazy::new(|| Mutex::new(LruCache::new(MAC_CACHE_SIZE)));

/// Node index prefix
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NodeCacheKey {
    id: HashValue,
    index: NodeIndex,
}
impl NodeCacheKey {
    pub fn new(accumulator_id: HashValue, index: NodeIndex) -> Self {
        Self {
            id: accumulator_id,
            index,
        }
    }
}
/// Accumulator global cache .
pub struct AccumulatorCache {}
impl AccumulatorCache {
    pub fn get_node(hash: HashValue) -> AccumulatorNode {
        match GLOBAL_NODE_CACHE.lock().get(&hash) {
            Some(node) => node.clone(),
            None => {
                error!("get node from cache error:{:}", hash);
                AccumulatorNode::new_empty()
            }
        }
    }
    pub fn get_node_hash(accumulator_id: HashValue, index: NodeIndex) -> HashValue {
        match GLOBAL_NODE_INDEX_CACHE
            .lock()
            .get(&NodeCacheKey::new(accumulator_id, index))
        {
            Some(node_hash) => node_hash.clone(),
            None => {
                error!("get node index hash error:{:?}", index);
                HashValue::zero()
            }
        }
    }
    pub fn save_node(node: AccumulatorNode) -> Result<()> {
        match GLOBAL_NODE_CACHE.lock().put(node.hash(), node.clone()) {
            Some(_) => Ok(()),
            None => {
                error!("save node cache error: {:?}", node);
                Ok(())
            }
        }
    }
    pub fn save_node_index(
        accumulator_id: HashValue,
        index: NodeIndex,
        node_hash: HashValue,
    ) -> Result<()> {
        match GLOBAL_NODE_INDEX_CACHE
            .lock()
            .put(NodeCacheKey::new(accumulator_id, index), node_hash)
        {
            Some(_) => Ok(()),
            None => {
                error!("save node index cache error: {:?}", index);
                Ok(())
            }
        }
    }

    pub fn save_nodes(nodes: Vec<AccumulatorNode>) -> Result<()> {
        let mut cache = GLOBAL_NODE_CACHE.lock();
        for node in nodes {
            match cache.put(node.hash(), node.clone()) {
                Some(_) => {}
                None => {
                    error!("save node cache error: {:?}", node);
                }
            }
        }
        Ok(())
    }

    pub fn save_node_indexes(accumulator_id: HashValue, nodes: Vec<AccumulatorNode>) -> Result<()> {
        let mut cache = GLOBAL_NODE_INDEX_CACHE.lock();
        for node in nodes {
            match cache.put(NodeCacheKey::new(accumulator_id, node.index()), node.hash()) {
                Some(_) => {}
                None => {
                    error!("save node index cache error: {:?}", node);
                }
            }
        }
        Ok(())
    }
}

pub struct MockAccumulatorStore {
    node_store: Mutex<HashMap<HashValue, AccumulatorNode>>,
}

impl MockAccumulatorStore {
    pub fn new() -> Self {
        Self {
            node_store: Mutex::new(HashMap::new()),
        }
    }
}

impl AccumulatorTreeStore for MockAccumulatorStore {}
impl AccumulatorReader for MockAccumulatorStore {
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        match self.node_store.lock().get(&hash) {
            Some(node) => Ok(Some(node.clone())),
            None => bail!("get node is null: {}", hash),
        }
    }

    fn multiple_get(&self, _hash_vec: Vec<HashValue>) -> Result<Vec<AccumulatorNode>, Error> {
        unimplemented!()
    }
}
impl AccumulatorWriter for MockAccumulatorStore {
    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.node_store.lock().insert(node.hash(), node);
        Ok(())
    }

    fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> Result<(), Error> {
        let mut store = self.node_store.lock();
        for node in nodes {
            store.insert(node.hash(), node);
        }
        Ok(())
    }

    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<(), Error> {
        for hash in node_hash_vec {
            self.node_store.lock().remove(&hash);
        }
        Ok(())
    }
}
