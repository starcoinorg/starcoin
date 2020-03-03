// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use logger::prelude::*;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};
use std::cell::RefCell;
use std::sync::Arc;

pub mod mock;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHash)]
pub struct StateProof {}

//this just a mock implement.
#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateNode {
    pub key: HashValue,
    pub value: Vec<u8>,
}

impl StateNode {
    pub fn new(key: HashValue, value: Vec<u8>) -> Self {
        return Self { key, value };
    }
}

impl CryptoHash for StateNode {
    fn crypto_hash(&self) -> HashValue {
        self.key
    }
}

pub trait StateNodeStore {
    fn get_node(&self, hash: &HashValue) -> Result<Option<StateNode>>;
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
        let mut bytes = self.root_hash.borrow().to_vec();
        bytes.extend_from_slice(value.as_slice());
        let new_root = HashValue::from_sha3_256(bytes.as_slice());
        let node = self
            .store
            .get_node(&hash)?
            .unwrap_or(StateNode::new(hash, value));
        self.store.save_node(node);
        self.root_hash.replace(new_root);
        //TODO
        Ok(())
    }

    pub fn root_hash(&self) -> HashValue {
        self.root_hash.borrow().clone()
    }

    pub fn get(&self, hash: &HashValue) -> Result<Option<Vec<u8>>> {
        Ok(self.store.get_node(hash)?.map(|node| node.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockStateNodeStore;
    use types::account_address::AccountAddress;

    #[stest::test]
    fn test_state_tree() {
        info!("test logger.");
        let state_store = Arc::new(MockStateNodeStore::new());
        let state_tree = StateTree::new(state_store, None);
        let old_root = state_tree.root_hash();
        let data = vec![1u8, 2u8, 3u8, 4u8];
        let address = AccountAddress::random();
        let hash = address.crypto_hash();
        state_tree.update(hash, data.clone()).unwrap();
        let data2 = state_tree.get(&hash).unwrap().unwrap();
        assert_eq!(data, data2);
        let new_root = state_tree.root_hash();
        assert_ne!(old_root, new_root);
    }
}
