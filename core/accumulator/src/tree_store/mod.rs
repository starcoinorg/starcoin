// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::node::AccumulatorStoreType;
use crate::node_index::NodeIndex;
use crate::{AccumulatorNode, AccumulatorReader, AccumulatorTreeStore, AccumulatorWriter};
use anyhow::{bail, Error, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use std::collections::HashMap;

/// Node index cache key.
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

pub type AccumulatorNodeKey = (HashValue, AccumulatorStoreType);

pub struct MockAccumulatorStore {
    node_store: Mutex<HashMap<AccumulatorNodeKey, AccumulatorNode>>,
}

impl MockAccumulatorStore {
    pub fn new() -> MockAccumulatorStore {
        MockAccumulatorStore {
            node_store: Mutex::new(HashMap::new()),
        }
    }
    pub fn get_store_key(store_type: AccumulatorStoreType, hash: HashValue) -> AccumulatorNodeKey {
        (hash, store_type)
    }
}

impl Default for MockAccumulatorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AccumulatorTreeStore for MockAccumulatorStore {}
impl AccumulatorReader for MockAccumulatorStore {
    fn get_node(
        &self,
        store_type: AccumulatorStoreType,
        hash: HashValue,
    ) -> Result<Option<AccumulatorNode>> {
        let key = Self::get_store_key(store_type, hash);
        let map = self.node_store.lock();
        match map.get(&key) {
            Some(node) => Ok(Some(node.clone())),
            None => bail!("get node is null: {}", hash),
        }
    }

    fn multiple_get(
        &self,
        _store_type: AccumulatorStoreType,
        _hash_vec: Vec<HashValue>,
    ) -> Result<Vec<AccumulatorNode>, Error> {
        unimplemented!()
    }
}
impl AccumulatorWriter for MockAccumulatorStore {
    fn save_node(&self, store_type: AccumulatorStoreType, node: AccumulatorNode) -> Result<()> {
        let key = Self::get_store_key(store_type, node.hash());
        self.node_store.lock().insert(key, node);
        Ok(())
    }

    fn save_nodes(
        &self,
        store_type: AccumulatorStoreType,
        nodes: Vec<AccumulatorNode>,
    ) -> Result<(), Error> {
        let mut store = self.node_store.lock();
        for node in nodes {
            store.insert(Self::get_store_key(store_type, node.hash()), node);
        }
        Ok(())
    }

    fn delete_nodes(
        &self,
        store_type: AccumulatorStoreType,
        node_hash_vec: Vec<HashValue>,
    ) -> Result<(), Error> {
        for hash in node_hash_vec {
            self.node_store
                .lock()
                .remove(&Self::get_store_key(store_type, hash));
        }
        Ok(())
    }
}
