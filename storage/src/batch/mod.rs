// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{ColumnFamilyName, KeyCodec, ValueCodec, WriteOp};
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Default, Clone)]
pub struct WriteBatch {
    pub rows: HashMap<ColumnFamilyName, BTreeMap<Vec<u8>, WriteOp>>,
}

impl WriteBatch {
    /// Creates an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an insert/update operation to the batch.
    pub fn put<K: KeyCodec, V: ValueCodec>(
        &mut self,
        prefix_name: ColumnFamilyName,
        key: K,
        value: V,
    ) -> Result<()> {
        let key = KeyCodec::encode_key(&key)?;
        let value = ValueCodec::encode_value(&value)?;
        self.rows
            .entry(prefix_name)
            .or_insert_with(BTreeMap::new)
            .insert(key, WriteOp::Value(value));

        Ok(())
    }

    /// Adds a delete operation to the batch.
    pub fn delete<K: KeyCodec>(&mut self, prefix_name: ColumnFamilyName, key: K) -> Result<()> {
        let key = KeyCodec::encode_key(&key)?;
        self.rows
            .entry(prefix_name)
            .or_insert_with(BTreeMap::new)
            .insert(key, WriteOp::Deletion);

        Ok(())
    }
}
