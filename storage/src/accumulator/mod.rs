// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::define_storage;
use crate::storage::{CodecStorage, KeyCodec, StorageInstance, ValueCodec};
use crate::{ensure_slice_len_eq, ACCUMULATOR_NODE_PREFIX_NAME};
use anyhow::Error;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use crypto::hash::HashValue;
use scs::SCSCodec;
use starcoin_accumulator::node_index::NodeIndex;
use starcoin_accumulator::{
    AccumulatorNode, AccumulatorReader, AccumulatorTreeStore, AccumulatorWriter,
};
use std::mem::size_of;
use std::sync::Arc;

define_storage!(
    AccumulatorNodeStore,
    HashValue,
    AccumulatorNode,
    ACCUMULATOR_NODE_PREFIX_NAME
);

pub struct AccumulatorStorage {
    node_store: AccumulatorNodeStore,
}

impl AccumulatorStorage {
    pub fn new(instance: StorageInstance) -> Self {
        let node_store = AccumulatorNodeStore::new(instance.clone());
        Self { node_store }
    }
}

impl KeyCodec for NodeIndex {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_inorder_index().to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<u64>())?;
        let index = (&data[..]).read_u64::<BigEndian>()?;
        Ok(NodeIndex::new(index))
    }
}

impl ValueCodec for AccumulatorNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl AccumulatorTreeStore for AccumulatorStorage {}
impl AccumulatorReader for AccumulatorStorage {
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        self.node_store.get(hash)
    }

    fn multiple_get(&self, _hash_vec: Vec<HashValue>) -> Result<Vec<AccumulatorNode>, Error> {
        unimplemented!()
    }
}

impl AccumulatorWriter for AccumulatorStorage {
    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.node_store.put(node.hash(), node)
    }

    fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> Result<(), Error> {
        let mut batch = WriteBatch::new();
        for node in nodes {
            batch
                .put(ACCUMULATOR_NODE_PREFIX_NAME, node.hash(), node)
                .unwrap();
        }
        self.node_store.write_batch(batch)
    }

    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<(), Error> {
        let mut batch = WriteBatch::new();
        for key in node_hash_vec {
            batch.delete(ACCUMULATOR_NODE_PREFIX_NAME, key).unwrap();
        }
        self.node_store.write_batch(batch)
    }
}
