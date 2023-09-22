// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::WriteOp;
use anyhow::Result;

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
