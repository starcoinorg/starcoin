// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[derive(Clone, Debug)]
pub struct RocksdbConfig {
    pub max_open_files: i32,
    pub max_total_wal_size: u64,
    pub wal_bytes_per_sync: u64,
    pub bytes_per_sync: u64,
}

impl RocksdbConfig {
    #[cfg(unix)]
    fn default_max_open_files() -> i32 {
        40960
    }

    #[cfg(windows)]
    fn default_max_open_files() -> i32 {
        256
    }

    pub fn new(
        max_open_files: i32,
        max_total_wal_size: u64,
        wal_bytes_per_sync: u64,
        bytes_per_sync: u64,
    ) -> Self {
        Self {
            max_open_files,
            max_total_wal_size,
            wal_bytes_per_sync,
            bytes_per_sync,
        }
    }
}

impl Default for RocksdbConfig {
    fn default() -> Self {
        Self {
            // Set max_open_files to 4096 instead of -1 to avoid keep-growing memory in accordance
            // with the number of files.
            max_open_files: Self::default_max_open_files(),
            // For now we set the max total WAL size to be 1G. This config can be useful when column
            // families are updated at non-uniform frequencies.
            max_total_wal_size: 1u64 << 30,
            // For sst table sync every size to be 1MB
            bytes_per_sync: 1u64 << 20,
            // For wal sync every size to be 1MB
            wal_bytes_per_sync: 1u64 << 20,
        }
    }
}
