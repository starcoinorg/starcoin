// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use starcoin_state_store_api::{StateNode, StateNodeStore};

use starcoin_crypto::HashValue;
use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;
#[derive(Default)]
pub struct MockStateNodeStore {
    nodes: RwLock<HashMap<HashValue, StateNode>>,
}

impl MockStateNodeStore {
    pub fn new() -> Self {
        Self::default()
        // instance.put(*SPARSE_MERKLE_PLACEHOLDER_HASH, Node::new_null().into());
    }

    pub fn all_nodes(&self) -> Vec<(HashValue, StateNode)> {
        let nodes = self.nodes.read().unwrap();
        nodes.iter().map(|(k, v)| (*k, v.clone())).collect()
    }
}

impl StateNodeStore for MockStateNodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        let nodes = self.nodes.read().unwrap();
        Ok(nodes.get(hash).cloned())
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        let mut nodes = self.nodes.write().unwrap();
        nodes.insert(key, node);
        Ok(())
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<(), Error> {
        let mut store_nodes = self.nodes.write().unwrap();
        store_nodes.extend(nodes.into_iter());
        // for (node_key, node) in nodes.iter() {
        //     self.put(*node_key, node.clone()).unwrap();
        // }
        Ok(())
    }
}
