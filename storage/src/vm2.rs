// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecKVStore, CodecWriteBatch};
use crate::{IntoSuper, Storage};
use starcoin_crypto::HashValue;
use starcoin_state_store_api::{StateNode, StateNodeStore};
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug)]
pub struct Storage2(pub Arc<Storage>);

impl StateNodeStore for Storage2 {
    fn get(&self, hash: &HashValue) -> anyhow::Result<Option<StateNode>> {
        self.deref().state_node_storage2.get(*hash)
    }

    fn put(&self, key: HashValue, node: StateNode) -> anyhow::Result<()> {
        self.deref().state_node_storage2.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> anyhow::Result<()> {
        let batch = CodecWriteBatch::new_puts(nodes.into_iter().collect());
        self.deref().state_node_storage2.write_batch(batch)
    }
}

pub trait Store2: StateNodeStore + IntoSuper<dyn StateNodeStore> {}

impl Store2 for Storage2 {}

impl Deref for Storage2 {
    type Target = Storage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
