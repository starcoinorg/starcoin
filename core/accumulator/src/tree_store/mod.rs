// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::node_index::NodeIndex;
use crate::AccumulatorNode;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use std::any::type_name;

pub mod mock;

pub trait AccumulatorTreeStore: std::marker::Send + std::marker::Sync {
    fn store_type(&self) -> &'static str {
        type_name::<Self>()
    }

    ///get node by node hash
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>>;
    /// multiple get nodes
    fn multiple_get(&self, hash_vec: Vec<HashValue>) -> Result<Vec<Option<AccumulatorNode>>>;

    /// save node
    fn save_node(&self, node: AccumulatorNode) -> Result<()>;
    /// batch save nodes
    fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> Result<()>;
    ///delete node
    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<()>;
}

/// Node index cache key.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NodeCacheKey {
    id: HashValue,
    index: NodeIndex,
}
impl NodeCacheKey {
    pub fn new(accumulator_id: HashValue, index: NodeIndex) -> Self {
        Self {
            id: accumulator_id,
            index,
        }
    }
}
