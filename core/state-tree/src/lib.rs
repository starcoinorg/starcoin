// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use logger::prelude::*;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHash)]
pub struct StateProof {}

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHash)]
pub struct StateNode {
    pub value: Vec<u8>,
}

pub trait StateNodeStore {
    fn get_node(&self, hash: HashValue) -> Result<Option<StateNode>>;
    fn save_node(&self, node: StateNode) -> Result<()>;
}

pub struct StateTree {
    store: Arc<dyn StateNodeStore>,
    root_hash: RefCell<HashValue>,
}

impl StateTree {
    pub fn new(store: Arc<dyn StateNodeStore>, root_hash: Option<HashValue>) -> Self {
        //TODO
        let root_hash = root_hash.unwrap_or(HashValue::zero());
        Self {
            store,
            root_hash: RefCell::new(root_hash),
        }
    }

    pub fn update(&self, hash: HashValue, value: Vec<u8>) -> Result<()> {
        let mut node = self.store.get_node(hash)?.unwrap_or_default();
        node.value = value;
        self.store.save_node(node);
        //TODO
        Ok(())
    }

    pub fn root_hash(&self) -> HashValue {
        self.root_hash.borrow().clone()
    }

    pub fn get(&self, hash: HashValue) -> Result<Option<Vec<u8>>> {
        Ok(self.store.get_node(hash)?.map(|node| node.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[stest::test]
    fn test_state_tree() {
        info!("test logger.");
    }
}
