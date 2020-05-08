// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::metrics::record_metrics;
use crate::storage::{ColumnFamilyName, InnerStore, WriteOp};
use crate::VEC_PREFIX_NAME;
use anyhow::{ensure, format_err, Error, Result};
use rocksdb::{WriteBatch as DBWriteBatch, WriteOptions, DB};
use std::collections::HashSet;
use std::path::Path;

pub struct DBStorage {
    db: DB,
}

impl DBStorage {
    pub fn new<P: AsRef<Path> + Clone>(db_root_path: P) -> Self {
        let path = db_root_path.as_ref().join("starcoindb");
        Self::open(path.clone(), false).expect("Unable to open StarcoinDB")
    }
    pub fn open(path: impl AsRef<Path>, readonly: bool) -> Result<Self> {
        let column_families = VEC_PREFIX_NAME.to_vec();
        {
            let cfs_set: HashSet<_> = column_families.iter().collect();
            ensure!(
                cfs_set.len() == column_families.len(),
                "Duplicate column family name found.",
            );
        }

        let db = if readonly {
            Self::open_readonly(path, column_families)?
        } else {
            let mut db_opts = rocksdb::Options::default();
            db_opts.create_if_missing(true);
            db_opts.create_missing_column_families(true);
            // For now we set the max total WAL size to be 1G. This config can be useful when column
            // families are updated at non-uniform frequencies.
            db_opts.set_max_total_wal_size(1 << 30);
            Self::open_inner(&db_opts, path, column_families)?
        };

        Ok(DBStorage { db })
    }

    fn open_inner(
        opts: &rocksdb::Options,
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

    fn open_readonly(path: impl AsRef<Path>, column_families: Vec<ColumnFamilyName>) -> Result<DB> {
        let db_opts = rocksdb::Options::default();
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
        for cf in &VEC_PREFIX_NAME.to_vec() {
            self.db.drop_cf(cf)?;
        }
        Ok(())
    }

    /// Flushes all memtable data. This is only used for testing `get_approximate_sizes_cf` in unit
    /// tests.
    pub fn flush_all(&self) -> Result<()> {
        for cf_name in VEC_PREFIX_NAME.to_vec() {
            let cf_handle = self.get_cf_handle(cf_name)?;
            self.db.flush_cf(cf_handle)?;
        }
        Ok(())
    }

    fn _db_exists(path: &Path) -> bool {
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
        record_metrics("db", prefix_name, "put").end_with(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            let result =
                self.db
                    .put_cf_opt(cf_handle, &key, &value, &Self::default_write_options())?;
            Ok(result)
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
    fn write_batch(&self, batch: WriteBatch) -> Result<()> {
        record_metrics("db", "", "write_batch").end_with(|| {
            let mut db_batch = DBWriteBatch::default();
            let cf_handle = self.get_cf_handle(batch.get_prefix_name())?;
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
