// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::define_storage;
use crate::storage::{CodecKVStore, ValueCodec};
use crate::StorageInstance;
use crate::{BLOCK_ACCUMULATOR_NODE_PREFIX_NAME, TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME};
use anyhow::Result;
use bcs_ext::BCSCodec;
use crypto::hash::HashValue;
use starcoin_accumulator::{AccumulatorNode, AccumulatorTreeStore};

define_storage!(
    BlockAccumulatorStorage,
    HashValue,
    AccumulatorNode,
    BLOCK_ACCUMULATOR_NODE_PREFIX_NAME
);

define_storage!(
    TransactionAccumulatorStorage,
    HashValue,
    AccumulatorNode,
    TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME
);

impl ValueCodec for AccumulatorNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

#[derive(Clone)]
pub struct AccumulatorStorage<S>
where
    S: CodecKVStore<HashValue, AccumulatorNode>,
{
    store: S,
}

impl AccumulatorStorage<BlockAccumulatorStorage> {
    pub fn new_block_accumulator_storage(
        instance: StorageInstance,
    ) -> AccumulatorStorage<BlockAccumulatorStorage> {
        Self {
            store: BlockAccumulatorStorage::new(instance),
        }
    }
}

impl AccumulatorStorage<TransactionAccumulatorStorage> {
    pub fn new_transaction_accumulator_storage(
        instance: StorageInstance,
    ) -> AccumulatorStorage<TransactionAccumulatorStorage> {
        Self {
            store: TransactionAccumulatorStorage::new(instance),
        }
    }
}

impl<S> AccumulatorTreeStore for AccumulatorStorage<S>
where
    S: CodecKVStore<HashValue, AccumulatorNode>,
{
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        self.store.get(hash)
    }

    fn multiple_get(&self, keys: Vec<HashValue>) -> Result<Vec<Option<AccumulatorNode>>> {
        self.store.multiple_get(keys)
    }

    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.store.put(node.hash(), node)
    }

    fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> Result<()> {
        self.store
            .put_all(nodes.into_iter().map(|node| (node.hash(), node)).collect())
    }

    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<()> {
        self.store.delete_all(node_hash_vec)
    }
}
