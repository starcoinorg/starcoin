// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

mod version;

use crate::{error::StorageInitError, metrics::StorageMetrics, ColumnFamilyName, WriteOp};
use anyhow::{ensure, format_err, Error, Result};
use rocksdb::{
    DBIterator, IteratorMode, Options, ReadOptions, WriteBatch as DBWriteBatch, WriteOptions, DB,
};
use starcoin_config::{check_open_fds_limit, RocksdbConfig};
use std::collections::HashSet;
use std::path::Path;
pub use version::*;

const RES_FDS: u64 = 4096;

#[allow(clippy::upper_case_acronyms)]
pub struct DBStorage {
    // Todo, make me private to other crates
    pub db: DB,
    cfs: Vec<ColumnFamilyName>,
    // Todo, make me private to other crates
    pub metrics: Option<StorageMetrics>,
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
            column_families,
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
            self.flush_cf(cf_name)?
        }
        Ok(())
    }

    pub fn flush_cf(&self, cf_name: &str) -> Result<()> {
        let cf_handle = self.get_cf_handle(cf_name)?;
        Ok(self.db.flush_cf(cf_handle)?)
    }

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
    pub fn list_cf(path: impl AsRef<Path>) -> Result<Vec<String>, Error> {
        Ok(rocksdb::DB::list_cf(&rocksdb::Options::default(), path)?)
    }

    fn db_exists(path: &Path) -> bool {
        let rocksdb_current_file = path.join("CURRENT");
        rocksdb_current_file.is_file()
    }

    pub fn get_cf_handle(&self, cf_name: &str) -> Result<&rocksdb::ColumnFamily> {
        self.db.cf_handle(cf_name).ok_or_else(|| {
            format_err!(
                "DB::cf_handle not found for column family name: {}",
                cf_name
            )
        })
    }

    pub fn default_write_options() -> WriteOptions {
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
        if config.parallelism > 1 {
            db_opts.increase_parallelism(config.parallelism as i32);
        }
        // cache
        // let cache = Cache::new_lru_cache(2 * 1024 * 1024 * 1024);
        // db_opts.set_row_cache(&cache.unwrap());
        db_opts
    }

    pub fn raw_iterator_cf_opt(
        &self,
        prefix_name: &str,
        mode: IteratorMode,
        readopts: ReadOptions,
    ) -> Result<DBIterator> {
        let cf_handle = self.get_cf_handle(prefix_name)?;
        Ok(self.db.iterator_cf_opt(cf_handle, readopts, mode))
    }

    pub fn sync_write_options() -> WriteOptions {
        let mut opts = WriteOptions::new();
        opts.set_sync(true);
        opts
    }
}
