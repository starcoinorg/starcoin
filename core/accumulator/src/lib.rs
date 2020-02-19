// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use crypto::{hash::CryptoHash, HashValue};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccumulatorProof {}

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccumulatorNode {}

pub trait AccumulatorStore {
    fn get_node(&self, hash: HashValue) -> Result<AccumulatorNode>;
    fn save_node(&self, node: AccumulatorNode) -> Result<()>;
}

pub struct Accumulator {
    store: Arc<dyn AccumulatorStore>,
    root_hash: HashValue,
}

impl Accumulator {
    pub fn new(store: Arc<dyn AccumulatorStore>, root_hash: HashValue) -> Self {
        Self { store, root_hash }
    }

    pub fn append(&self, leaves: &[HashValue]) -> Result<()> {
        //TODO
        Ok(())
    }

    pub fn root_hash(&self) -> HashValue {
        self.root_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulator() {}
}
