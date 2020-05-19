// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::define_storage;
use crate::storage::{CodecStorage, ValueCodec};
use crate::STATE_NODE_PREFIX_NAME;
use anyhow::{Error, Result};
use crypto::HashValue;
use forkable_jellyfish_merkle::node_type::Node;
use starcoin_state_store_api::{StateNode, StateNodeStore};
use std::collections::BTreeMap;
use std::sync::Arc;

define_storage!(StateStorage, HashValue, StateNode, STATE_NODE_PREFIX_NAME);

impl ValueCodec for StateNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.0.encode()
    }
    #[allow(clippy::all)]
    fn decode_value(data: &[u8]) -> Result<Self> {
        Node::decode(data).map(|n| StateNode(n))
    }
}

impl StateNodeStore for StateStorage {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        //TODO use ref as key
        self.store.get(*hash)
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.store.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<(), Error> {
        let mut batch = WriteBatch::new();
        for (key, node) in nodes.iter() {
            batch.put(*key, node.clone())?;
        }
        self.store.write_batch(batch)
    }
}
