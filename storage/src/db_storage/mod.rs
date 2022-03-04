// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::errors::StorageInitError;
use crate::metrics::{record_metrics, StorageMetrics};
use crate::storage::{ColumnFamilyName, InnerStore, KeyCodec, ValueCodec, WriteOp};
use crate::{StorageVersion, DEFAULT_PREFIX_NAME};
use anyhow::{ensure, format_err, Error, Result};
use rocksdb::{Options, ReadOptions, WriteBatch as DBWriteBatch, WriteOptions, DB};
use starcoin_config::{check_open_fds_limit, RocksdbConfig};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::path::Path;

const RES_FDS: u64 = 4096;

#[allow(clippy::upper_case_acronyms)]
pub struct DBStorage {
    db: DB,
    cfs: Vec<ColumnFamilyName>,
    metrics: Option<StorageMetrics>,
}

impl DBStorage {
    pub fn new<P: AsRef<Path> + Clone>(
        db_root_path: P,
        rocksdb_config: RocksdbConfig,
        metrics: Option<StorageMetrics>,
    ) -> Result<Self> {
        //TODO find a compat way to remove the `starcoindb` path
        let path = db_root_path.as_ref().join("starcoindb");
        Self::open_with_cfs(
            path,
            StorageVersion::current_version()
                .get_column_family_names()
                .to_vec(),
            false,
            rocksdb_config,
            metrics,
        )
    }

    pub fn open_with_cfs(
        root_path: impl AsRef<Path>,
        column_families: Vec<ColumnFamilyName>,
        readonly: bool,
        rocksdb_config: RocksdbConfig,
        metrics: Option<StorageMetrics>,
    ) -> Result<Self> {
        let path = root_path.as_ref();

        let cfs_set: HashSet<_> = column_families.iter().collect();
        {
            ensure!(
                cfs_set.len() == column_families.len(),
                "Duplicate column family name found.",
            );
        }
        if Self::db_exists(path) {
            let cf_vec = Self::list_cf(path)?;
            let mut db_cfs_set: HashSet<_> = cf_vec.iter().collect();
            db_cfs_set.remove(&DEFAULT_PREFIX_NAME.to_string());
            ensure!(
                db_cfs_set.len() <= cfs_set.len(),
                StorageInitError::StorageCheckError(format_err!(
                    "ColumnFamily in db ({:?}) not same as ColumnFamily in code {:?}.",
                    column_families,
                    cf_vec
                ))
            );
            let mut remove_cf_vec = Vec::new();
            db_cfs_set.iter().for_each(|k| {
                if !cfs_set.contains(&k.as_str()) {
                    remove_cf_vec.push(<&std::string::String>::clone(k));
                }
            });
            ensure!(
                remove_cf_vec.is_empty(),
                StorageInitError::StorageCheckError(format_err!(
                    "Can not remove ColumnFamily, ColumnFamily in db ({:?}) not in code {:?}.",
                    remove_cf_vec,
                    cf_vec
                ))
            );
        }

        let mut rocksdb_opts = Self::gen_rocksdb_options(&rocksdb_config);

        let db = if readonly {
            Self::open_readonly(&rocksdb_opts, path, column_families.clone())?
        } else {
            rocksdb_opts.create_if_missing(true);
            rocksdb_opts.create_missing_column_families(true);
            Self::open_inner(&rocksdb_opts, path, column_families.clone())?
        };
        check_open_fds_limit(rocksdb_config.max_open_files as u64 + RES_FDS)?;
        Ok(DBStorage {
            db,
            cfs: column_families,
            metrics,
        })
    }

    fn open_inner(
        opts: &Options,
        path: impl AsRef<Path>,
        column_families: Vec<ColumnFamilyName>,
    ) -> Result<DB> {
        let inner = rocksdb::DB::open_cf_descriptors(
            opts,
            path,
            column_families.iter().map(|cf_name| {
                let mut cf_opts = rocksdb::Options::default();
                cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
                /*
                cf_opts.set_compression_per_level(&[
                    rocksdb::DBCompressionType::None,
                    rocksdb::DBCompressionType::None,
                    rocksdb::DBCompressionType::Lz4,
                    rocksdb::DBCompressionType::Lz4,
                    rocksdb::DBCompressionType::Lz4,
                    rocksdb::DBCompressionType::Lz4,
                    rocksdb::DBCompressionType::Lz4,
                ]);
                */
                rocksdb::ColumnFamilyDescriptor::new((*cf_name).to_string(), cf_opts)
            }),
        )?;
        Ok(inner)
    }

    fn open_readonly(
        db_opts: &Options,
        path: impl AsRef<Path>,
        column_families: Vec<ColumnFamilyName>,
    ) -> Result<DB> {
        let error_if_log_file_exists = false;
        let inner = rocksdb::DB::open_cf_for_read_only(
            db_opts,
            path,
            &column_families,
            error_if_log_file_exists,
        )?;
        Ok(inner)
    }

    pub fn drop_cf(&mut self) -> Result<(), Error> {
        for cf in self.cfs.clone() {
            self.db.drop_cf(cf)?;
        }
        Ok(())
    }

    pub fn drop_unused_cfs(&mut self, names: Vec<&str>) -> Result<(), Error> {
        // https://github.com/facebook/rocksdb/issues/1295
        for name in names {
            for cf in &self.cfs {
                if cf == &name {
                    self.db.drop_cf(name)?;
                    let opt = Options::default();
                    self.db.create_cf(name, &opt)?;
                    break;
                }
            }
        }
        Ok(())
    }

    /// Flushes all memtable data. This is only used for testing `get_approximate_sizes_cf` in unit
    /// tests.
    pub fn flush_all(&self) -> Result<()> {
        for cf_name in &self.cfs {
            let cf_handle = self.get_cf_handle(cf_name)?;
            self.db.flush_cf(cf_handle)?;
        }
        Ok(())
    }

    /// List cf
    pub fn list_cf(path: impl AsRef<Path>) -> Result<Vec<String>, Error> {
        Ok(rocksdb::DB::list_cf(&rocksdb::Options::default(), path)?)
    }

    fn db_exists(path: &Path) -> bool {
        let rocksdb_current_file = path.join("CURRENT");
        rocksdb_current_file.is_file()
    }

    fn get_cf_handle(&self, cf_name: &str) -> Result<&rocksdb::ColumnFamily> {
        self.db.cf_handle(cf_name).ok_or_else(|| {
            format_err!(
                "DB::cf_handle not found for column family name: {}",
                cf_name
            )
        })
    }

    fn default_write_options() -> WriteOptions {
        let mut opts = WriteOptions::new();
        opts.set_sync(false);
        opts
    }

    fn gen_rocksdb_options(config: &RocksdbConfig) -> Options {
        let mut db_opts = Options::default();
        db_opts.set_max_open_files(config.max_open_files);
        db_opts.set_max_total_wal_size(config.max_total_wal_size);
        db_opts.set_wal_bytes_per_sync(config.wal_bytes_per_sync);
        db_opts.set_bytes_per_sync(config.bytes_per_sync);
        // db_opts.enable_statistics();
        // write buffer size
        db_opts.set_max_write_buffer_number(5);
        db_opts.set_max_background_jobs(5);
        // cache
        // let cache = Cache::new_lru_cache(2 * 1024 * 1024 * 1024);
        // db_opts.set_row_cache(&cache.unwrap());
        db_opts
    }
    fn iter_with_direction<K, V>(
        &self,
        prefix_name: &str,
        direction: ScanDirection,
    ) -> Result<SchemaIterator<K, V>>
    where
        K: KeyCodec,
        V: ValueCodec,
    {
        let cf_handle = self.get_cf_handle(prefix_name)?;
        Ok(SchemaIterator::new(
            self.db
                .raw_iterator_cf_opt(cf_handle, ReadOptions::default()),
            direction,
        ))
    }

    /// Returns a forward [`SchemaIterator`] on a certain schema.
    pub fn iter<K, V>(&self, prefix_name: &str) -> Result<SchemaIterator<K, V>>
    where
        K: KeyCodec,
        V: ValueCodec,
    {
        self.iter_with_direction(prefix_name, ScanDirection::Forward)
    }

    /// Returns a backward [`SchemaIterator`] on a certain schema.
    pub fn rev_iter<K, V>(&self, prefix_name: &str) -> Result<SchemaIterator<K, V>>
    where
        K: KeyCodec,
        V: ValueCodec,
    {
        self.iter_with_direction(prefix_name, ScanDirection::Backward)
    }

    fn sync_write_options() -> WriteOptions {
        let mut opts = WriteOptions::new();
        opts.set_sync(true);
        opts
    }
}

pub enum ScanDirection {
    Forward,
    Backward,
}

pub struct SchemaIterator<'a, K, V> {
    db_iter: rocksdb::DBRawIterator<'a>,
    direction: ScanDirection,
    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
}

impl<'a, K, V> SchemaIterator<'a, K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    fn new(db_iter: rocksdb::DBRawIterator<'a>, direction: ScanDirection) -> Self {
        SchemaIterator {
            db_iter,
            direction,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        }
    }

    /// Seeks to the first key.
    pub fn seek_to_first(&mut self) {
        self.db_iter.seek_to_first();
    }

    /// Seeks to the last key.
    pub fn seek_to_last(&mut self) {
        self.db_iter.seek_to_last();
    }

    /// Seeks to the first key whose binary representation is equal to or greater than that of the
    /// `seek_key`.
    pub fn seek(&mut self, seek_key: Vec<u8>) -> Result<()> {
        self.db_iter.seek(&seek_key);
        Ok(())
    }

    /// Seeks to the last key whose binary representation is less than or equal to that of the
    /// `seek_key`.
    pub fn seek_for_prev(&mut self, seek_key: Vec<u8>) -> Result<()> {
        self.db_iter.seek_for_prev(&seek_key);
        Ok(())
    }

    fn next_impl(&mut self) -> Result<Option<(K, V)>> {
        if !self.db_iter.valid() {
            self.db_iter.status()?;
            return Ok(None);
        }

        let raw_key = self.db_iter.key().expect("Iterator must be valid.");
        let raw_value = self.db_iter.value().expect("Iterator must be valid.");
        let key = K::decode_key(raw_key)?;
        let value = V::decode_value(raw_value)?;
        match self.direction {
            ScanDirection::Forward => self.db_iter.next(),
            ScanDirection::Backward => self.db_iter.prev(),
        }

        Ok(Some((key, value)))
    }
}

impl<'a, K, V> Iterator for SchemaIterator<'a, K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    type Item = Result<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().transpose()
    }
}

impl InnerStore for DBStorage {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        record_metrics("db", prefix_name, "get", self.metrics.as_ref()).call(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            let result = self.db.get_cf(cf_handle, key.as_slice())?;
            Ok(result)
        })
    }

    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        if let Some(metrics) = self.metrics.as_ref() {
            metrics
                .storage_item_bytes
                .with_label_values(&[prefix_name])
                .observe((key.len() + value.len()) as f64);
        }

        record_metrics("db", prefix_name, "put", self.metrics.as_ref()).call(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            self.db
                .put_cf_opt(cf_handle, &key, &value, &Self::default_write_options())?;
            Ok(())
        })
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        record_metrics("db", prefix_name, "contains_key", self.metrics.as_ref()).call(|| match self
            .get(prefix_name, key)
        {
            Ok(Some(_)) => Ok(true),
            _ => Ok(false),
        })
    }
    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
        record_metrics("db", prefix_name, "remove", self.metrics.as_ref()).call(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            self.db.delete_cf(cf_handle, &key)?;
            Ok(())
        })
    }

    /// Writes a group of records wrapped in a WriteBatch.
    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        record_metrics("db", prefix_name, "write_batch", self.metrics.as_ref()).call(|| {
            let mut db_batch = DBWriteBatch::default();
            let cf_handle = self.get_cf_handle(prefix_name)?;
            for (key, write_op) in &batch.rows {
                match write_op {
                    WriteOp::Value(value) => db_batch.put_cf(cf_handle, key, value),
                    WriteOp::Deletion => db_batch.delete_cf(cf_handle, key),
                };
            }
            self.db
                .write_opt(db_batch, &Self::default_write_options())?;
            Ok(())
        })
    }

    fn get_len(&self) -> Result<u64> {
        unimplemented!()
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>> {
        unimplemented!()
    }

    fn put_sync(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        if let Some(metrics) = self.metrics.as_ref() {
            metrics
                .storage_item_bytes
                .with_label_values(&[prefix_name])
                .observe((key.len() + value.len()) as f64);
        }

        record_metrics("db", prefix_name, "put_sync", self.metrics.as_ref()).call(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            self.db
                .put_cf_opt(cf_handle, &key, &value, &Self::sync_write_options())?;
            Ok(())
        })
    }

    fn write_batch_sync(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        record_metrics("db", prefix_name, "write_batch_sync", self.metrics.as_ref()).call(|| {
            let mut db_batch = DBWriteBatch::default();
            let cf_handle = self.get_cf_handle(prefix_name)?;
            for (key, write_op) in &batch.rows {
                match write_op {
                    WriteOp::Value(value) => db_batch.put_cf(cf_handle, key, value),
                    WriteOp::Deletion => db_batch.delete_cf(cf_handle, key),
                };
            }
            self.db.write_opt(db_batch, &Self::sync_write_options())?;
            Ok(())
        })
    }
}
