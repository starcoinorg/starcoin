// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Port selected RocksDB options for tuning underlying rocksdb instance of DiemDB.
/// see https://github.com/facebook/rocksdb/blob/master/include/rocksdb/options.h
/// for detailed explanations.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RocksdbConfig {
    pub max_open_files: i32,
    pub max_total_wal_size: u64,
}

impl Default for RocksdbConfig {
    #[cfg(not(windows))]
    fn default() -> Self {
        Self {
            // Set max_open_files to 10k instead of -1 to avoid keep-growing memory in accordance
            // with the number of files.
            max_open_files: 2_000,
            // For now we set the max total WAL size to be 1G. This config can be useful when column
            // families are updated at non-uniform frequencies.
            max_total_wal_size: 1u64 << 30,
        }
    }
    #[cfg(windows)]
    fn default() -> Self {
        Self {
            // For the windows system et max_open_files to 256 instead of -1 to avoid keep-growing
            // memory in accordance with the number of files.
            max_open_files: 256,
            // For now we set the max total WAL size to be 1G. This config can be useful when column
            // families are updated at non-uniform frequencies.
            max_total_wal_size: 1u64 << 30,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    dir: PathBuf,
    #[serde(skip)]
    absolute_dir: Option<PathBuf>,
    /// Rocksdb-specific configurations
    #[serde(default)]
    pub rocksdb_config: RocksdbConfig,
}

impl StorageConfig {
    pub fn dir(&self) -> PathBuf {
        self.absolute_dir
            .as_ref()
            .cloned()
            .expect("config should init first.")
    }
}

impl ConfigModule for StorageConfig {
    fn default_with_opt(_opt: &StarcoinOpt, _base: &BaseConfig) -> Result<Self> {
        Ok(Self {
            dir: PathBuf::from("starcoindb/db"),
            absolute_dir: None,
            rocksdb_config: RocksdbConfig::default(),
        })
    }

    fn after_load(&mut self, _opt: &StarcoinOpt, base: &BaseConfig) -> Result<()> {
        self.absolute_dir = Some(if self.dir.is_relative() {
            base.data_dir().join(&self.dir)
        } else {
            self.dir.clone()
        });
        Ok(())
    }
}
