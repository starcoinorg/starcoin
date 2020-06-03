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
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::node_index::NodeIndex;
use starcoin_accumulator::{
    AccumulatorNode, AccumulatorReader, AccumulatorTreeStore, AccumulatorWriter,
};
use std::io::Write;
use std::mem::size_of;
use std::sync::Arc;

define_storage!(
    AccumulatorNodeStore,
    AccumulatorNodeKey,
    AccumulatorNode,
    ACCUMULATOR_NODE_PREFIX_NAME
);

pub type AccumulatorNodeKey = (HashValue, AccumulatorStoreType);

pub struct AccumulatorStorage {
    node_store: AccumulatorNodeStore,
}

impl AccumulatorStorage {
    pub fn new(instance: StorageInstance) -> Self {
        let node_store = AccumulatorNodeStore::new(instance);
        Self { node_store }
    }
    pub fn get_store_key(store_type: AccumulatorStoreType, hash: HashValue) -> AccumulatorNodeKey {
        (hash, store_type)
    }
}

impl KeyCodec for AccumulatorNodeKey {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (hash, store_type) = self.clone();

        let mut encoded_key = Vec::with_capacity(size_of::<AccumulatorNodeKey>());
        encoded_key.write_all(&hash.to_vec())?;
        encoded_key.write_all(&store_type.encode()?)?;
        Ok(encoded_key)
    }

    fn decode_key(data: &[u8]) -> Result<Self, Error> {
        let hash = HashValue::from_slice(&data[..HashValue::LENGTH])?;
        let store_type = AccumulatorStoreType::decode(&data[HashValue::LENGTH..])?;
        Ok((hash, store_type))
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
    fn get_node(
        &self,
        store_type: AccumulatorStoreType,
        hash: HashValue,
    ) -> Result<Option<AccumulatorNode>> {
        self.node_store.get(Self::get_store_key(store_type, hash))
    }

    fn multiple_get(
        &self,
        _store_type: AccumulatorStoreType,
        _hash_vec: Vec<HashValue>,
    ) -> Result<Vec<AccumulatorNode>, Error> {
        unimplemented!()
    }
}

impl AccumulatorWriter for AccumulatorStorage {
    fn save_node(&self, store_type: AccumulatorStoreType, node: AccumulatorNode) -> Result<()> {
        self.node_store
            .put(Self::get_store_key(store_type, node.hash()), node)
    }

    fn save_nodes(
        &self,
        store_type: AccumulatorStoreType,
        nodes: Vec<AccumulatorNode>,
    ) -> Result<(), Error> {
        let mut batch = WriteBatch::new();
        for node in nodes {
            batch.put(Self::get_store_key(store_type.clone(), node.hash()), node)?;
        }
        self.node_store.write_batch(batch)
    }

    fn delete_nodes(
        &self,
        store_type: AccumulatorStoreType,
        node_hash_vec: Vec<HashValue>,
    ) -> Result<(), Error> {
        let mut batch = WriteBatch::new();
        for key in node_hash_vec {
            batch.delete(Self::get_store_key(store_type.clone(), key))?;
        }
        self.node_store.write_batch(batch)
    }
}
