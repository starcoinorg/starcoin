// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use crypto::{hash::CryptoHash, HashValue};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateProof {}

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateNode {}

pub trait StateStore {
    fn get_node(&self, hash: HashValue) -> Result<StateNode>;
    fn save_node(&self, node: StateNode) -> Result<()>;
}

pub struct SparseMerkleTree {
    store: Arc<dyn StateStore>,
    root_hash: HashValue,
}

impl SparseMerkleTree {
    pub fn new(store: Arc<dyn StateStore>, root_hash: HashValue) -> Self {
        Self { store, root_hash }
    }

    pub fn update(&self, hash: HashValue, value: Vec<u8>) -> Result<Self> {
        unimplemented!()
    }

    pub fn root_hash(&self) -> HashValue {
        self.root_hash
    }

    pub fn get(&self, hash: HashValue) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_tree() {}
}
