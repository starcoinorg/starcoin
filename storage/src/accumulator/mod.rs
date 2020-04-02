// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::define_storage;
use crate::storage::{CodecStorage, KeyCodec, StorageInstance, ValueCodec};
use crate::{ensure_slice_len_eq, ACCUMULATOR_INDEX_PREFIX_NAME, ACCUMULATOR_NODE_PREFIX_NAME};
use anyhow::Error;
use anyhow::{bail, ensure, Result};
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
    AccumulatorIndexStore,
    NodeIndex,
    HashValue,
    ACCUMULATOR_INDEX_PREFIX_NAME
);

define_storage!(
    AccumulatorNodeStore,
    HashValue,
    AccumulatorNode,
    ACCUMULATOR_NODE_PREFIX_NAME
);

pub struct AccumulatorStorage {
    index_store: AccumulatorIndexStore,
    node_store: AccumulatorNodeStore,
}

impl AccumulatorStorage {
    pub fn new(instance: StorageInstance) -> Self {
        let index_store = AccumulatorIndexStore::new(instance.clone());
        let node_store = AccumulatorNodeStore::new(instance.clone());
        Self {
            index_store,
            node_store,
        }
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
    fn get(&self, index: NodeIndex) -> Result<Option<AccumulatorNode>, Error> {
        let node_index = self.index_store.get(index).unwrap();
        match node_index {
            Some(hash) => self.node_store.get(hash),
            None => bail!("get accumulator node index is null {:?}", node_index),
        }
    }

    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        self.node_store.get(hash)
    }
}

impl AccumulatorWriter for AccumulatorStorage {
    fn save(&self, index: NodeIndex, hash: HashValue) -> Result<(), Error> {
        self.index_store.put(index, hash)
    }

    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.node_store.put(node.hash(), node)
    }

    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<(), Error> {
        let mut batch = WriteBatch::new();
        for key in node_hash_vec {
            batch.delete(ACCUMULATOR_NODE_PREFIX_NAME, key).unwrap();
        }
        self.node_store.write_batch(batch)
    }

    fn delete_nodes_index(&self, vec_index: Vec<NodeIndex>) -> Result<(), Error> {
        ensure!(
            vec_index.len() > 0,
            " invalid index len : {}.",
            vec_index.len()
        );
        let mut batch = WriteBatch::new();
        for index in vec_index {
            batch.delete(ACCUMULATOR_INDEX_PREFIX_NAME, index).unwrap();
        }
        self.index_store.write_batch(batch)
    }
}
