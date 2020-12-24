// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::errors::StorageInitError;
use crate::metrics::{record_metrics, STORAGE_ITER_BYTES};
use crate::storage::{ColumnFamilyName, InnerStore, WriteOp};
use crate::{DEFAULT_PREFIX_NAME, VEC_PREFIX_NAME};
use anyhow::{ensure, format_err, Error, Result};
use rocksdb::{Options, WriteBatch as DBWriteBatch, WriteOptions, DB};
use starcoin_config::RocksdbConfig;
use std::collections::HashSet;
use std::path::Path;

pub struct DBStorage {
    db: DB,
    cfs: Vec<ColumnFamilyName>,
}

impl DBStorage {
    pub fn new<P: AsRef<Path> + Clone>(
        db_root_path: P,
        rocksdb_config: RocksdbConfig,
    ) -> Result<Self> {
        let path = db_root_path.as_ref().join("starcoindb");
        Self::open_with_cfs(path, VEC_PREFIX_NAME.to_vec(), false, rocksdb_config)
    }

    pub fn open_with_cfs(
        root_path: impl AsRef<Path>,
        column_families: Vec<ColumnFamilyName>,
        readonly: bool,
        rocksdb_config: RocksdbConfig,
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

        Ok(DBStorage {
            db,
            cfs: column_families,
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
            &db_opts,
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
        opts.set_sync(true);
        opts
    }

    fn gen_rocksdb_options(config: &RocksdbConfig) -> Options {
        let mut db_opts = Options::default();
        db_opts.set_max_open_files(config.max_open_files);
        db_opts.set_max_total_wal_size(config.max_total_wal_size);
        db_opts
    }
}

impl InnerStore for DBStorage {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        record_metrics("db", prefix_name, "get").end_with(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            let result = self.db.get_cf(cf_handle, key.as_slice())?;
            Ok(result)
        })
    }

    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        STORAGE_ITER_BYTES
            .with_label_values(&[prefix_name])
            .observe((key.len() + value.len()) as f64);
        record_metrics("db", prefix_name, "put").end_with(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            self.db
                .put_cf_opt(cf_handle, &key, &value, &Self::default_write_options())?;
            Ok(())
        })
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        record_metrics("db", prefix_name, "contains_key").end_with(|| {
            match self.get(prefix_name, key) {
                Ok(Some(_)) => Ok(true),
                _ => Ok(false),
            }
        })
    }
    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
        record_metrics("db", prefix_name, "remove").end_with(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            self.db.delete_cf(cf_handle, &key)?;
            Ok(())
        })
    }

    /// Writes a group of records wrapped in a WriteBatch.
    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        record_metrics("db", "batch", prefix_name).end_with(|| {
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
}
