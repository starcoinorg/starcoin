// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::storage::{CodecStorage, Repository, ValueCodec};
use crate::STATE_NODE_PREFIX_NAME;
use anyhow::{Error, Result};
use crypto::HashValue;
use forkable_jellyfish_merkle::node_type::Node;
use state_tree::{StateNode, StateNodeStore};
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct StateNodeStorage {
    storage: CodecStorage<HashValue, StateNode>,
}

impl StateNodeStorage {
    pub fn new(storage: Arc<dyn Repository>) -> Self {
        Self {
            storage: CodecStorage::new(storage),
        }
    }
}

impl ValueCodec for StateNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.0.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Node::decode(data).map(|n| StateNode(n))
    }
}

impl StateNodeStore for StateNodeStorage {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        //TODO use ref as key
        self.storage.get(hash.clone())
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.storage.put(key, node)
    }

    fn write_batch(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<(), Error> {
        let mut batch = WriteBatch::new();
        for (key, node) in nodes.iter() {
            batch
                .put::<HashValue, StateNode>(STATE_NODE_PREFIX_NAME, *key, node.clone())
                .unwrap();
        }
        self.storage.write_batch(batch)
    }
}
