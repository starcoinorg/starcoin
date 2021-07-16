// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

/// Port selected RocksDB options for tuning underlying rocksdb instance of DiemDB.
/// see https://github.com/facebook/rocksdb/blob/master/include/rocksdb/options.h
/// for detailed explanations.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(default, deny_unknown_fields)]
pub struct RocksdbConfig {
    #[structopt(name = "rocksdb-max-open-files", long, help = "rocksdb max open files")]
    pub max_open_files: i32,
    #[structopt(
        name = "rocksdb-max-total-wal-sizes",
        long,
        help = "rocksdb max total WAL sizes"
    )]
    pub max_total_wal_size: u64,
}

impl RocksdbConfig {
    #[cfg(any(target_os = "macos"))]
    fn default_max_open_files() -> i32 {
        64
    }

    #[cfg(any(target_os = "linux"))]
    fn default_max_open_files() -> i32 {
        256
    }

    #[cfg(windows)]
    fn default_max_open_files() -> i32 {
        64
    }
}

impl Default for RocksdbConfig {
    fn default() -> Self {
        Self {
            // Set max_open_files to 256 instead of -1 to avoid keep-growing memory in accordance
            // with the number of files.
            max_open_files: Self::default_max_open_files(),
            // For now we set the max total WAL size to be 1G. This config can be useful when column
            // families are updated at non-uniform frequencies.
            max_total_wal_size: 1u64 << 30,
        }
    }
}

static DEFAULT_DB_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("starcoindb/db"));
pub const DEFAULT_CACHE_SIZE: usize = 20000;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "rocksdb-max-open-files", long, help = "rocksdb max open files")]
    pub max_open_files: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "rocksdb-max-total-wal-sizes",
        long,
        help = "rocksdb max total WAL sizes"
    )]
    pub max_total_wal_size: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "cache-sizes", long, help = "cache sizes")]
    pub cache_size: Option<usize>,

    #[serde(skip)]
    #[structopt(skip)]
    base: Option<Arc<BaseConfig>>,
}

impl StorageConfig {
    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn dir(&self) -> PathBuf {
        self.base().data_dir().join(DEFAULT_DB_DIR.as_path())
    }

    pub fn rocksdb_config(&self) -> RocksdbConfig {
        let default = RocksdbConfig::default();
        RocksdbConfig {
            max_open_files: self.max_open_files.unwrap_or(default.max_open_files),
            max_total_wal_size: self
                .max_total_wal_size
                .unwrap_or(default.max_total_wal_size),
        }
    }
    pub fn cache_size(&self) -> usize {
        self.cache_size.unwrap_or(DEFAULT_CACHE_SIZE)
    }
}

impl ConfigModule for StorageConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);
        if opt.storage.max_open_files.is_some() {
            self.max_open_files = opt.storage.max_open_files;
        }
        if opt.storage.max_total_wal_size.is_some() {
            self.max_total_wal_size = opt.storage.max_total_wal_size;
        }
        if opt.storage.cache_size.is_some() {
            self.cache_size = opt.storage.cache_size;
        }
        Ok(())
    }
}
