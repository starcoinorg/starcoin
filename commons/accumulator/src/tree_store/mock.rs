// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{NodeIndex, AccumulatorNode};
use crate::tree_store::AccumulatorTreeStore_tmp;
use anyhow::{bail, Result};
use parking_lot::Mutex;
use starcoin_crypto::HashValue;
use std::collections::HashMap;

pub struct MockAccumulatorStore {
    node_store: Mutex<HashMap<NodeIndex, HashValue>>,
}

impl MockAccumulatorStore {
    pub fn new() -> MockAccumulatorStore {
        MockAccumulatorStore {
            node_store: Mutex::new(HashMap::new()),
        }
    }
    pub fn copy_from(&self) -> Self {
        Self {
            node_store: Mutex::new(self.node_store.lock().clone()),
        }
    }
}

impl Default for MockAccumulatorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AccumulatorTreeStore_tmp for MockAccumulatorStore {
    fn get_node(&self, index: NodeIndex) -> Result<Option<HashValue>> {
        let map = self.node_store.lock();
        match map.get(&index) {
            Some(node) => Ok(Some(node.clone())),
            None => bail!("get node is null: {:?}", index),
        }
    }

    fn multiple_get(&self, _hash_vec: Vec<NodeIndex>) -> Result<Vec<Option<HashValue>>> {
        unimplemented!()
    }

    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.node_store.lock().insert(node.index(), node.hash());
        Ok(())
    }

    fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> Result<()> {
        let mut store = self.node_store.lock();
        for node in nodes {
            store.insert(node.index(), node.hash());
        }
        Ok(())
    }

    fn delete_nodes(&self, node_index_vec: Vec<NodeIndex>) -> Result<()> {
        for index in node_index_vec {
            self.node_store.lock().remove(&index);
        }
        Ok(())
    }
}
