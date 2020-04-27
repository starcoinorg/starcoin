// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    AccumulatorNode, AccumulatorReader, AccumulatorTreeStore, AccumulatorWriter, GLOBAL_NODE_CACHE,
};
use anyhow::{bail, Error, Result};
use logger::prelude::*;
use starcoin_crypto::HashValue;
use std::collections::HashMap;
use std::sync::Mutex;
pub struct AccumulatorCache {}
impl AccumulatorCache {
    pub fn get_node(hash: HashValue) -> Result<AccumulatorNode> {
        match GLOBAL_NODE_CACHE.lock().get(&hash) {
            Some(node) => Ok(node.clone()),
            None => {
                error!("get node from cache error:{:}", hash);
                Ok(AccumulatorNode::new_empty())
            }
        }
    }
}

pub struct MockAccumulatorStore {
    node_store: Mutex<HashMap<HashValue, AccumulatorNode>>,
}

impl MockAccumulatorStore {
    pub fn new() -> Self {
        Self {
            node_store: Mutex::new(HashMap::new()),
        }
    }
}

impl AccumulatorTreeStore for MockAccumulatorStore {}
impl AccumulatorReader for MockAccumulatorStore {
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        match self.node_store.lock().unwrap().get(&hash) {
            Some(node) => Ok(Some(node.clone())),
            None => bail!("get node is null: {}", hash),
        }
    }

    fn multiple_get(&self, _hash_vec: Vec<HashValue>) -> Result<Vec<AccumulatorNode>, Error> {
        unimplemented!()
    }
}
impl AccumulatorWriter for MockAccumulatorStore {
    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.node_store.lock().unwrap().insert(node.hash(), node);
        Ok(())
    }

    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<(), Error> {
        for hash in node_hash_vec {
            self.node_store.lock().unwrap().remove(&hash);
        }
        Ok(())
    }
}
