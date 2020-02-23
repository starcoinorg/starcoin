// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecStorage, Repository, ValueCodec};
use anyhow::Result;
use crypto::{hash::CryptoHash, HashValue};
use scs::SCSCodec;
use state_tree::{StateNode, StateNodeStore};
use std::sync::Arc;

pub struct StateNodeStorage {
    storage: CodecStorage<HashValue, StateNode>,
}

const KEY_PREFIX: &str = "StateNode";

impl StateNodeStorage {
    pub fn new(storage: Arc<dyn Repository>) -> Self {
        Self {
            storage: CodecStorage::new(storage, KEY_PREFIX),
        }
    }
}

impl ValueCodec for StateNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl StateNodeStore for StateNodeStorage {
    fn get_node(&self, hash: HashValue) -> Result<Option<StateNode>> {
        self.storage.get(hash)
    }

    fn save_node(&self, node: StateNode) -> Result<()> {
        let key = node.crypto_hash();
        self.storage.put(key, node)
    }
}
