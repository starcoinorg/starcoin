// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{ColumnFamilyName, KeyCodec, ValueCodec, WriteOp};
use crate::DEFAULT_PREFIX_NAME;
use anyhow::Result;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct WriteBatch {
    prefix_name: ColumnFamilyName,
    pub rows: BTreeMap<Vec<u8>, WriteOp>,
}

impl WriteBatch {
    /// Creates an empty batch.
    pub fn new() -> Self {
        Self {
            prefix_name: DEFAULT_PREFIX_NAME,
            rows: BTreeMap::new(),
        }
    }

    /// Create an prefix_name batch.
    pub fn new_with_name(prefix_name: ColumnFamilyName) -> Self {
        Self {
            prefix_name,
            rows: BTreeMap::new(),
        }
    }

    /// Get prefix_name.
    pub fn get_prefix_name(&self) -> ColumnFamilyName {
        self.prefix_name
    }

    /// Adds an insert/update operation to the batch.
    pub fn put<K: KeyCodec, V: ValueCodec>(&mut self, key: K, value: V) -> Result<()> {
        let key = KeyCodec::encode_key(&key)?;
        let value = ValueCodec::encode_value(&value)?;
        self.rows.insert(key, WriteOp::Value(value));
        Ok(())
    }

    /// Adds a delete operation to the batch.
    pub fn delete<K: KeyCodec>(&mut self, key: K) -> Result<()> {
        let key = KeyCodec::encode_key(&key)?;
        self.rows.insert(key, WriteOp::Deletion);
        Ok(())
    }
}
