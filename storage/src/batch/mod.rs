// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecWriteBatch, KeyCodec, ValueCodec, WriteOp};
use anyhow::Result;
use std::convert::TryFrom;

pub type WriteBatch = GWriteBatch<Vec<u8>, Vec<u8>>;

#[derive(Debug, Default, Clone)]
pub struct GWriteBatch<K, V> {
    pub rows: Vec<WriteOp<K, V>>,
}

impl<K: Default, V: Default> GWriteBatch<K, V> {
    /// Creates an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_rows(rows: Vec<WriteOp<K, V>>) -> Self {
        Self { rows }
    }

    /// Adds an insert/update operation to the batch.
    pub fn put(&mut self, key: K, value: V) -> Result<()> {
        self.rows.push(WriteOp::Value(key, value));
        Ok(())
    }

    /// Adds a delete operation to the batch.
    pub fn delete(&mut self, key: K) -> Result<()> {
        self.rows.push(WriteOp::Deletion(key));
        Ok(())
    }

    ///Clear all operation to the next batch.
    pub fn clear(&mut self) -> Result<()> {
        self.rows.clear();
        Ok(())
    }
}

fn into_raw_op<K: KeyCodec, V: ValueCodec>(op: WriteOp<K, V>) -> Result<WriteOp<Vec<u8>, Vec<u8>>> {
    Ok(match op {
        WriteOp::Value(k, v) => WriteOp::Value(k.encode_key()?, v.encode_value()?),
        WriteOp::Deletion(k) => WriteOp::Deletion(k.encode_key()?),
    })
}

impl<K, V> TryFrom<CodecWriteBatch<K, V>> for WriteBatch
where
    K: KeyCodec,
    V: ValueCodec,
{
    type Error = anyhow::Error;

    fn try_from(batch: CodecWriteBatch<K, V>) -> Result<Self, Self::Error> {
        let rows: Result<Vec<WriteOp<Vec<u8>, Vec<u8>>>> =
            batch.into_iter().map(|op| into_raw_op(op)).collect();
        Ok(WriteBatch::new_with_rows(rows?))
    }
}
