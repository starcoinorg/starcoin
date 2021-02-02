// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use parking_lot::RwLock;
use starcoin_crypto::HashValue;
use starcoin_state_store_api::{StateNode, StateNodeStore};
use std::collections::{BTreeMap, HashMap};

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
        let nodes = self.nodes.read();
        nodes.iter().map(|(k, v)| (*k, v.clone())).collect()
    }
}

impl StateNodeStore for MockStateNodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        let nodes = self.nodes.read();
        Ok(nodes.get(hash).cloned())
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        let mut nodes = self.nodes.write();
        nodes.insert(key, node);
        Ok(())
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<(), Error> {
        let mut store_nodes = self.nodes.write();
        store_nodes.extend(nodes.into_iter());
        Ok(())
    }
}
