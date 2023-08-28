// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub mod db;
pub mod error;
pub mod iterator;
pub mod metrics;
pub mod schema;

use crate::db::DBStorage;
use crate::error::{StoreError, StoreResult};
use crate::iterator::{ScanDirection, SchemaIterator};
use crate::metrics::StorageMetrics;
use crate::schema::{KeyCodec, Schema, ValueCodec};
use parking_lot::Mutex;
use rocksdb::{DBIterator, IteratorMode, ReadOptions};
use starcoin_config::RocksdbConfig;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

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

    pub fn put<S: Schema>(&self, key: &S::Key, val: &S::Value) -> Result<(), StoreError> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let value = <S::Value as ValueCodec<S>>::encode_value(val)?;
        self.rows
            .lock()
            .entry(S::COLUMN_FAMILY)
            .or_insert_with(Vec::new)
            .push(WriteOp::Value(key, value));

        Ok(())
    }

    pub fn delete<S: Schema>(&self, key: &S::Key) -> Result<(), StoreError> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;

        self.rows
            .lock()
            .entry(S::COLUMN_FAMILY)
            .or_insert_with(Vec::new)
            .push(WriteOp::Deletion(key));

        Ok(())
    }
}

#[derive(Clone)]
pub struct DB {
    pub name: String, // for logging
    pub inner: Arc<DBStorage>,
}

impl DB {
    pub fn create_from_path(
        name: &str,
        root_path: impl AsRef<Path>,
        column_families: Vec<ColumnFamilyName>,
        readonly: bool,
        rocksdb_config: RocksdbConfig,
        metrics: Option<StorageMetrics>,
    ) -> StoreResult<Self> {
        let db_storage = DBStorage::open_with_cfs(
            root_path,
            column_families,
            readonly,
            rocksdb_config,
            metrics,
        )?;

        Ok(DB {
            name: name.to_owned(),
            inner: Arc::new(db_storage),
        })
    }

    pub fn write_schemas(&self, batch: SchemaBatch) -> Result<(), StoreError> {
        let rows_locked = batch.rows.lock();

        for row in rows_locked.iter() {
            self.inner
                .write_batch_inner(row.0, row.1, false /*normal write*/)?
        }

        Ok(())
    }

    pub fn get<S: Schema>(&self, key: &S::Key) -> Result<Option<S::Value>, StoreError> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let cf_handle = self.inner.get_cf_handle(S::COLUMN_FAMILY)?;
        self.inner
            .db
            .get_cf(cf_handle, raw_key)
            .map_err(Into::into)
            .and_then(|raw_value| {
                raw_value
                    .map(|v| <S::Value as ValueCodec<S>>::decode_value(&v))
                    .transpose()
            })
    }

    pub fn put<S: Schema>(&self, key: &S::Key, value: &S::Value) -> Result<(), StoreError> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let raw_value = <S::Value as ValueCodec<S>>::encode_value(value)?;
        let cf_handle = self.inner.get_cf_handle(S::COLUMN_FAMILY)?;

        self.inner.db.put_cf(cf_handle, raw_key, raw_value)?;

        Ok(())
    }

    pub fn remove<S: Schema>(&self, key: &S::Key) -> Result<(), StoreError> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let cf_handle = self.inner.get_cf_handle(S::COLUMN_FAMILY)?;

        self.inner.db.delete_cf(cf_handle, raw_key)?;
        Ok(())
    }

    pub fn flush_cf(&self, cf_name: &str) -> Result<(), StoreError> {
        Ok(self.inner.flush_cf(cf_name)?)
    }

    pub fn iterator_cf_opt<S: Schema>(
        &self,
        mode: IteratorMode,
        readopts: ReadOptions,
    ) -> Result<DBIterator, StoreError> {
        Ok(self
            .inner
            .raw_iterator_cf_opt(S::COLUMN_FAMILY, mode, readopts)?)
    }

    pub fn iter_with_direction<S: Schema>(
        &self,
        direction: ScanDirection,
    ) -> Result<SchemaIterator<S>, StoreError> {
        let cf_handle = self.inner.get_cf_handle(S::COLUMN_FAMILY)?;
        Ok(SchemaIterator::new(
            self.inner
                .db
                .raw_iterator_cf_opt(cf_handle, ReadOptions::default()),
            direction,
        ))
    }
}
