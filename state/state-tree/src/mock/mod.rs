// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{StateNode, StateNodeStore};
use anyhow::{Error, Result};

use starcoin_crypto::HashValue;
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

pub struct MockStateNodeStore {
    nodes: Mutex<HashMap<HashValue, StateNode>>,
}

impl MockStateNodeStore {
    pub fn new() -> Self {
        let instance = Self {
            nodes: Mutex::new(HashMap::new()),
        };
        // instance.put(*SPARSE_MERKLE_PLACEHOLDER_HASH, Node::new_null().into());
        instance
    }

    pub fn all_nodes(&self) -> Vec<(HashValue, StateNode)> {
        let nodes = self.nodes.lock().unwrap();
        nodes.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

impl StateNodeStore for MockStateNodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        let nodes = self.nodes.lock().unwrap();
        Ok(nodes.get(hash).cloned())
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(key, node);
        Ok(())
    }

    fn write_batch(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<(), Error> {
        for (node_key, node) in nodes.iter() {
            self.put(*node_key, node.clone()).unwrap();
        }
        Ok(())
    }
}
