// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

mod version;

use crate::iterator::ScanDirection;
use crate::{
    error::StorageInitError,
    metrics::StorageMetrics,
    schema::{KeyCodec, Schema, ValueCodec},
    ColumnFamilyName, SchemaBatch, SchemaIterator, WriteOp,
};
use anyhow::{ensure, format_err, Error, Result};
use rocksdb::{Options, ReadOptions, WriteBatch as DBWriteBatch, WriteOptions, DB};
use starcoin_config::{check_open_fds_limit, RocksdbConfig};
use std::collections::HashSet;
use std::path::Path;
pub use version::*;

const RES_FDS: u64 = 4096;

#[allow(clippy::upper_case_acronyms)]
pub struct DBStorage {
    name: String, // for logging
    db: DB,
    cfs: Vec<ColumnFamilyName>,
    _metrics: Option<StorageMetrics>,
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
            "default",
            path,
            StorageVersion::current_version()
                .get_column_family_names()
                .to_vec(),
            false,
            rocksdb_config,
            metrics,
        )
    }

    pub fn open_with_cfs<P>(
        name: &str,
        root_path: P,
        column_families: Vec<ColumnFamilyName>,
        readonly: bool,
        rocksdb_config: RocksdbConfig,
        metrics: Option<StorageMetrics>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Self::open_with_cfs_as_secondary(
            name,
            root_path,
            None,
            column_families,
            readonly,
            rocksdb_config,
            metrics,
        )
    }

    fn open_with_cfs_as_secondary<P>(
        name: &str,
        primary_path: P,
        secondary_path: Option<P>,
        column_families: Vec<ColumnFamilyName>,
        readonly: bool,
        rocksdb_config: RocksdbConfig,
        metrics: Option<StorageMetrics>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = primary_path.as_ref();

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
                    remove_cf_vec.push(<&String>::clone(k));
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
            if let Some(secondary_path) = secondary_path {
                Self::open_inner(
                    &rocksdb_opts,
                    path,
                    Some(secondary_path.as_ref()),
                    column_families.clone(),
                )?
            } else {
                Self::open_readonly_inner(&rocksdb_opts, path, column_families.clone())?
            }
        } else {
            rocksdb_opts.create_if_missing(true);
            rocksdb_opts.create_missing_column_families(true);
            Self::open_inner(&rocksdb_opts, path, None, column_families.clone())?
        };
        check_open_fds_limit(rocksdb_config.max_open_files as u64 + RES_FDS)?;
        Ok(DBStorage {
            name: name.to_string(),
            db,
            cfs: column_families,
            _metrics: metrics,
        })
    }

    fn open_inner<P>(
        opts: &Options,
        primary_path: P,
        secondary_path: Option<P>,
        column_families: Vec<ColumnFamilyName>,
    ) -> Result<DB>
    where
        P: AsRef<Path>,
    {
        let cfs = column_families.iter().map(|cf_name| {
            let mut cf_opts = Options::default();
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
        });

        let inner = if let Some(secondary_path) = secondary_path {
            DB::open_cf_descriptors_as_secondary(opts, primary_path, secondary_path, cfs)
        } else {
            DB::open_cf_descriptors(opts, primary_path, cfs)
        };

        Ok(inner?)
    }

    fn open_readonly_inner(
        db_opts: &Options,
        path: impl AsRef<Path>,
        column_families: Vec<ColumnFamilyName>,
    ) -> Result<DB> {
        let error_if_log_file_exists = false;
        let inner =
            DB::open_cf_for_read_only(db_opts, path, column_families, error_if_log_file_exists)?;
        Ok(inner)
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
            self.flush_cf(cf_name)?
        }
        Ok(())
    }

    pub fn flush_cf(&self, cf_name: &str) -> Result<()> {
        let cf_handle = self.get_cf_handle(cf_name)?;
        Ok(self.db.flush_cf(cf_handle)?)
    }

    // todo: make me private
    pub fn write_batch_inner(&self, prefix_name: &str, rows: &[WriteOp], sync: bool) -> Result<()> {
        let mut db_batch = DBWriteBatch::default();
        let cf_handle = self.get_cf_handle(prefix_name)?;
        for write_op in rows {
            match write_op {
                WriteOp::Value(key, value) => db_batch.put_cf(cf_handle, key, value),
                WriteOp::Deletion(key) => db_batch.delete_cf(cf_handle, key),
            };
        }

        let write_opts = if sync {
            Self::sync_write_options()
        } else {
            Self::default_write_options()
        };

        self.db.write_opt(db_batch, &write_opts)?;
        Ok(())
    }

    /// List cf
    fn list_cf(path: impl AsRef<Path>) -> Result<Vec<String>, Error> {
        Ok(DB::list_cf(&Options::default(), path)?)
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
        // todo: configure parallelism for backend rocksdb
        //if config.parallelism > 1 {
        //    db_opts.increase_parallelism(config.parallelism as i32);
        //}
        // cache
        // let cache = Cache::new_lru_cache(2 * 1024 * 1024 * 1024);
        // db_opts.set_row_cache(&cache.unwrap());
        db_opts
    }

    fn sync_write_options() -> WriteOptions {
        let mut opts = WriteOptions::new();
        opts.set_sync(true);
        opts
    }
}

// The new Apis
impl DBStorage {
    pub fn open(
        name: &str,
        root_path: impl AsRef<Path>,
        column_families: Vec<ColumnFamilyName>,
        rocksdb_config: RocksdbConfig,
        metrics: Option<StorageMetrics>,
    ) -> Result<Self> {
        Self::open_with_cfs(
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
        Self::open_with_cfs(
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
        DBStorage::open_with_cfs_as_secondary(
            name,
            primary_path,
            Some(secondary_path),
            column_families,
            true,
            rocksdb_config,
            metrics,
        )
    }

    pub fn write_schemas(&self, batch: SchemaBatch) -> Result<()> {
        let rows_locked = batch.rows.lock();

        for row in rows_locked.iter() {
            self.write_batch_inner(row.0, row.1, false /*normal write*/)?
        }

        Ok(())
    }

    pub fn get<S: Schema>(&self, key: &S::Key) -> Result<Option<S::Value>> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY)?;
        self.db
            .get_pinned_cf(cf_handle, raw_key)
            .map_err(Into::into)
            .and_then(|raw_value| {
                raw_value
                    .map(|v| <S::Value as ValueCodec<S>>::decode_value(&v))
                    .transpose()
            })
    }

    pub fn multi_get<S: Schema>(&self, keys: &[S::Key]) -> Result<Vec<Option<S::Value>>> {
        let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY)?;
        let keys = keys
            .iter()
            .map(|key| <S::Key as KeyCodec<S>>::encode_key(key))
            .collect::<Result<Vec<_>>>()?;

        self.db
            .batched_multi_get_cf(cf_handle, keys, false)
            .into_iter()
            .map(|result| {
                result.map_err(Into::into).and_then(|raw| {
                    raw.map(|v| <S::Value as ValueCodec<S>>::decode_value(&v))
                        .transpose()
                })
            })
            .collect::<Result<Vec<_>>>()
    }

    pub fn put<S: Schema>(&self, key: &S::Key, value: &S::Value) -> Result<()> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let raw_value = <S::Value as ValueCodec<S>>::encode_value(value)?;
        let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY)?;

        self.db.put_cf(cf_handle, raw_key, raw_value)?;

        Ok(())
    }

    pub fn remove<S: Schema>(&self, key: &S::Key) -> Result<()> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY)?;

        self.db.delete_cf(cf_handle, raw_key)?;
        Ok(())
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
        let cf_handle = self.get_cf_handle(S::COLUMN_FAMILY)?;
        Ok(SchemaIterator::new(
            self.db.raw_iterator_cf_opt(cf_handle, opts),
            direction,
        ))
    }

    pub fn get_property(&self, cf_name: &str, property_name: &str) -> Result<u64> {
        self.db
            .property_int_value_cf(self.get_cf_handle(cf_name)?, property_name)?
            .ok_or_else(|| {
                format_err!(
                    "Unable to get property \"{}\" of  column family \"{}\" in db \"{}\".",
                    property_name,
                    cf_name,
                    self.name,
                )
            })
    }
}

// FixMe: Remove these functions
impl DBStorage {
    pub fn get_no_schema(&self, cf_name: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let cf = self.get_cf_handle(cf_name)?;
        self.db.get_cf(cf, key).map_err(Into::into)
    }

    pub fn put_no_schema(&self, cf_name: &str, key: &[u8], value: &[u8]) -> Result<()> {
        let cf = self.get_cf_handle(cf_name)?;
        self.db.put_cf(cf, key, value).map_err(Into::into)
    }

    pub fn put_no_schema_opt(
        &self,
        cf_name: &str,
        key: &[u8],
        value: &[u8],
        opts: &WriteOptions,
    ) -> Result<()> {
        let cf = self.get_cf_handle(cf_name)?;
        self.db.put_cf_opt(cf, key, value, opts).map_err(Into::into)
    }

    pub fn contains_key(&self, cf_name: &str, key: &[u8]) -> Result<bool> {
        self.get_no_schema(cf_name, key).map(|s| s.is_some())
    }

    pub fn remove_no_schema(&self, cf_name: &str, key: &[u8]) -> Result<()> {
        let cf_handle = self.get_cf_handle(cf_name)?;
        self.db.delete_cf(cf_handle, key).map_err(Into::into)
    }

    pub fn multi_get_no_schema(
        &self,
        cf_name: &str,
        keys: &[Vec<u8>],
    ) -> Result<Vec<Option<Vec<u8>>>> {
        let cf_handle = self.get_cf_handle(cf_name)?;
        self.db
            .batched_multi_get_cf(cf_handle, keys, false)
            .into_iter()
            .map(|result| {
                result
                    .map_err(Into::into)
                    .map(|raw| raw.map(|v| v.to_vec()))
            })
            .collect::<Result<Vec<_>>>()
    }

    pub fn iter_no_schema(
        &self,
        cf_name: &str,
        opts: rocksdb::ReadOptions,
        mode: rocksdb::IteratorMode,
    ) -> Result<rocksdb::DBIterator> {
        let cf_handle = self.get_cf_handle(cf_name)?;
        Ok(self.db.iterator_cf_opt(cf_handle, opts, mode))
    }
}
