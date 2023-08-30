// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub mod db;
pub mod error;
pub mod iterator;
pub mod metrics;
pub mod schema;

use crate::{
    db::DBStorage,
    iterator::{ScanDirection, SchemaIterator},
    metrics::StorageMetrics,
    schema::{KeyCodec, Schema, ValueCodec},
};
use anyhow::{format_err, Result};
use parking_lot::Mutex;
use rocksdb::ReadOptions;
use starcoin_config::RocksdbConfig;
use std::{collections::HashMap, path::Path, sync::Arc};

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
    ) -> Result<Self> {
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

    pub fn open(
        name: &str,
        root_path: impl AsRef<Path>,
        column_families: Vec<ColumnFamilyName>,
        rocksdb_config: RocksdbConfig,
        metrics: Option<StorageMetrics>,
    ) -> Result<Self> {
        Self::create_from_path(
            name,
            root_path,
            column_families,
            false,
            rocksdb_config,
            metrics,
        )
    }

    pub fn open_readonly(
        name: &str,
        root_path: impl AsRef<Path>,
        column_families: Vec<ColumnFamilyName>,
        rocksdb_config: RocksdbConfig,
        metrics: Option<StorageMetrics>,
    ) -> Result<Self> {
        Self::create_from_path(
            name,
            root_path,
            column_families,
            true,
            rocksdb_config,
            metrics,
        )
    }

    pub fn open_cf_as_secondary<P>(
        name: &str,
        primary_path: P,
        secondary_path: P,
        column_families: Vec<ColumnFamilyName>,
        rocksdb_config: RocksdbConfig,
        metrics: Option<StorageMetrics>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let db_storage = DBStorage::open_with_cfs_as_secondary(
            primary_path,
            Some(secondary_path),
            column_families,
            true,
            rocksdb_config,
            metrics,
        )?;

        Ok(DB {
            name: name.to_owned(),
            inner: Arc::new(db_storage),
        })
    }

    pub fn write_schemas(&self, batch: SchemaBatch) -> Result<()> {
        let rows_locked = batch.rows.lock();

        for row in rows_locked.iter() {
            self.inner
                .write_batch_inner(row.0, row.1, false /*normal write*/)?
        }

        Ok(())
    }

    pub fn get<S: Schema>(&self, key: &S::Key) -> Result<Option<S::Value>> {
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

    pub fn put<S: Schema>(&self, key: &S::Key, value: &S::Value) -> Result<()> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let raw_value = <S::Value as ValueCodec<S>>::encode_value(value)?;
        let cf_handle = self.inner.get_cf_handle(S::COLUMN_FAMILY)?;

        self.inner.db.put_cf(cf_handle, raw_key, raw_value)?;

        Ok(())
    }

    pub fn remove<S: Schema>(&self, key: &S::Key) -> Result<()> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let cf_handle = self.inner.get_cf_handle(S::COLUMN_FAMILY)?;

        self.inner.db.delete_cf(cf_handle, raw_key)?;
        Ok(())
    }

    pub fn flush_cf(&self, cf_name: &str) -> Result<()> {
        self.inner.flush_cf(cf_name)
    }

    pub fn iter<S: Schema>(&self, opts: ReadOptions) -> Result<SchemaIterator<S>> {
        self.iter_with_direction(opts, ScanDirection::Forward)
    }

    pub fn rev_iter<S: Schema>(&self, opts: ReadOptions) -> Result<SchemaIterator<S>> {
        self.iter_with_direction(opts, ScanDirection::Backward)
    }

    fn iter_with_direction<S: Schema>(
        &self,
        opts: ReadOptions,
        direction: ScanDirection,
    ) -> Result<SchemaIterator<S>> {
        let cf_handle = self.inner.get_cf_handle(S::COLUMN_FAMILY)?;
        Ok(SchemaIterator::new(
            self.inner.db.raw_iterator_cf_opt(cf_handle, opts),
            direction,
        ))
    }

    pub fn get_property(&self, cf_name: &str, property_name: &str) -> Result<u64> {
        self.inner
            .db
            .property_int_value_cf(self.inner.get_cf_handle(cf_name)?, property_name)?
            .ok_or_else(|| {
                format_err!(
                    "Unable to get property \"{}\" of  column family \"{}\".",
                    property_name,
                    cf_name,
                )
            })
    }
}
