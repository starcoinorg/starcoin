// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::storage::{CodecStorage, InnerRepository, KeyCodec, Repository, Storage, ValueCodec};
use crate::{ensure_slice_len_eq, ACCUMULATOR_INDEX_PREFIX_NAME, ACCUMULATOR_NODE_PREFIX_NAME};
use anyhow::Error;
use anyhow::{bail, ensure, Result};
use byteorder::{BigEndian, ReadBytesExt};
use crypto::hash::HashValue;
use scs::SCSCodec;
use starcoin_accumulator::node_index::NodeIndex;
use starcoin_accumulator::{
    AccumulatorNode, AccumulatorNodeReader, AccumulatorNodeStore, AccumulatorNodeWriter,
};
use std::mem::size_of;
use std::sync::Arc;

pub struct AccumulatorStore {
    index_storage: CodecStorage<NodeIndex, HashValue>,
    node_store: CodecStorage<HashValue, AccumulatorNode>,
}

impl AccumulatorStore {
    pub fn new(storage: Arc<dyn Repository>) -> Self {
        Self {
            index_storage: CodecStorage::new(storage.clone()),
            node_store: CodecStorage::new(storage.clone()),
        }
    }
    pub fn two_new(
        cache_storage: Arc<dyn InnerRepository>,
        db_storage: Arc<dyn InnerRepository>,
    ) -> Self {
        Self {
            index_storage: CodecStorage::new(Arc::new(Storage::new(
                cache_storage.clone(),
                db_storage.clone(),
                ACCUMULATOR_INDEX_PREFIX_NAME,
            ))),
            node_store: CodecStorage::new(Arc::new(Storage::new(
                cache_storage.clone(),
                db_storage.clone(),
                ACCUMULATOR_NODE_PREFIX_NAME,
            ))),
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

impl AccumulatorNodeStore for AccumulatorStore {}
impl AccumulatorNodeReader for AccumulatorStore {
    fn get(&self, index: NodeIndex) -> Result<Option<AccumulatorNode>, Error> {
        let node_index = self.index_storage.get(index).unwrap();
        match node_index {
            Some(hash) => self.node_store.get(hash),
            None => bail!("get accumulator node index is null {:?}", node_index),
        }
    }

    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        self.node_store.get(hash)
    }
}

impl AccumulatorNodeWriter for AccumulatorStore {
    fn save(&self, index: NodeIndex, hash: HashValue) -> Result<(), Error> {
        self.index_storage.put(index, hash)
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
        self.index_storage.write_batch(batch)
    }
}
