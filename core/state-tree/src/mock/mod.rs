// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{StateNode, StateNodeStore};
use anyhow::Result;
use forkable_jellyfish_merkle::node_type::Node;
use starcoin_crypto::hash::CryptoHash;
use starcoin_crypto::HashValue;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct MockStateNodeStore {
    nodes: RefCell<HashMap<HashValue, StateNode>>,
}

impl MockStateNodeStore {
    pub fn new() -> Self {
        let instance = Self {
            nodes: RefCell::new(HashMap::new()),
        };
        instance.put(HashValue::zero(), Node::new_null().into());
        instance
    }
}

impl StateNodeStore for MockStateNodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        Ok(self.nodes.borrow().get(hash).cloned())
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.nodes.borrow_mut().insert(key, node);
        Ok(())
    }
}
