// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub mod db;
pub mod error;
pub mod iterator;
pub mod metrics;
pub mod schema;

use crate::{
    iterator::SchemaIterator,
    schema::{KeyCodec, Schema, ValueCodec},
};
use anyhow::Result;
use parking_lot::Mutex;
use std::collections::HashMap;

pub type ColumnFamilyName = &'static str;

#[derive(Debug, Clone)]
pub enum GWriteOp<K, V> {
    Value(K, V),
    Deletion(K),
}

pub type WriteOp = GWriteOp<Vec<u8>, Vec<u8>>;

#[derive(Debug)]
pub struct SchemaBatch {
    rows: Mutex<HashMap<ColumnFamilyName, Vec<WriteOp>>>,
}

impl Default for SchemaBatch {
    fn default() -> Self {
        Self {
            rows: Mutex::new(HashMap::new()),
        }
    }
}

impl SchemaBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_bunch(cf_name: ColumnFamilyName, batch: Vec<WriteOp>) -> Self {
        Self {
            rows: Mutex::new(HashMap::from([(cf_name, batch)])),
        }
    }

    pub fn put<S: Schema>(&self, key: &S::Key, val: &S::Value) -> Result<()> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let value = <S::Value as ValueCodec<S>>::encode_value(val)?;
        self.rows
            .lock()
            .entry(S::COLUMN_FAMILY)
            .or_insert_with(Vec::new)
            .push(WriteOp::Value(key, value));

        Ok(())
    }

    pub fn delete<S: Schema>(&self, key: &S::Key) -> Result<()> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;

        self.rows
            .lock()
            .entry(S::COLUMN_FAMILY)
            .or_insert_with(Vec::new)
            .push(WriteOp::Deletion(key));

        Ok(())
    }
}
