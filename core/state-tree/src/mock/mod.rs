// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{StateNode, StateNodeStore};
use anyhow::Result;
use starcoin_crypto::hash::CryptoHash;
use starcoin_crypto::HashValue;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct MockStateNodeStore {
    nodes: RefCell<HashMap<HashValue, StateNode>>,
}

impl MockStateNodeStore {
    pub fn new() -> Self {
        Self {
            nodes: RefCell::new(HashMap::new()),
        }
    }
}

impl StateNodeStore for MockStateNodeStore {
    fn get_node(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        Ok(self.nodes.borrow().get(hash).cloned())
    }

    fn save_node(&self, node: StateNode) -> Result<()> {
        let hash = node.crypto_hash();
        self.nodes.borrow_mut().insert(hash, node);
        Ok(())
    }
}
