// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BLOCK_ACCUMULATOR_NODE_PREFIX_NAME, TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_accumulator::{AccumulatorNode, AccumulatorTreeStore};
use starcoin_crypto::hash::HashValue;
use starcoin_schemadb::{
    db::DBStorage,
    define_schema,
    schema::{KeyCodec, Schema, ValueCodec},
    SchemaBatch,
};
use std::marker::PhantomData;
use std::sync::Arc;

pub(crate) type BlockAccumulatorStorage = AccumulatorStorage<BlockAccumulator>;
pub(crate) type TransactionAccumulatorStorage = AccumulatorStorage<TransactionAccumulator>;

define_schema!(
    BlockAccumulator,
    HashValue,
    AccumulatorNode,
    BLOCK_ACCUMULATOR_NODE_PREFIX_NAME
);

define_schema!(
    TransactionAccumulator,
    HashValue,
    AccumulatorNode,
    TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME
);

impl KeyCodec<BlockAccumulator> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<BlockAccumulator> for AccumulatorNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl KeyCodec<TransactionAccumulator> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<TransactionAccumulator> for AccumulatorNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

#[derive(Clone)]
pub struct AccumulatorStorage<S: Schema> {
    db: Arc<DBStorage>,
    _phantom: PhantomData<S>,
}

impl<S: Schema> AccumulatorStorage<S> {
    pub fn new_accumulator_storage(db: &Arc<DBStorage>) -> AccumulatorStorage<S> {
        Self {
            db: Arc::clone(db),
            _phantom: Default::default(),
        }
    }
}

macro_rules! impl_accumulator_tree_store_for_schema {
    ($schema: ty) => {
        impl AccumulatorTreeStore for AccumulatorStorage<$schema> {
            fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
                self.db.get::<$schema>(&hash)
            }

            fn multiple_get(&self, keys: Vec<HashValue>) -> Result<Vec<Option<AccumulatorNode>>> {
                self.db.batched_multi_get::<$schema>(&keys)
            }

            fn save_node(&self, node: AccumulatorNode) -> Result<()> {
                self.db.put::<$schema>(&node.hash(), &node)
            }

            fn save_nodes(&self, nodes: Vec<AccumulatorNode>) -> Result<()> {
                let batch = SchemaBatch::new();
                for node in nodes.iter() {
                    batch.put::<$schema>(&node.hash(), node)?;
                }
                self.db.write_schemas(batch)
            }

            fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<()> {
                let batch = SchemaBatch::new();
                for h in node_hash_vec.iter() {
                    batch.delete::<$schema>(h)?;
                }
                self.db.write_schemas(batch)
            }
        }
    };
}

impl_accumulator_tree_store_for_schema!(TransactionAccumulator);
impl_accumulator_tree_store_for_schema!(BlockAccumulator);
