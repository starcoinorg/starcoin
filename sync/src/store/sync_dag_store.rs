use std::{path::Path, sync::Arc};

use starcoin_config::{temp_dir, RocksdbConfig, StorageConfig};
use starcoin_storage::db_storage::DBStorage;

use super::sync_absent_ancestor::{SyncAbsentBlockStore, SYNC_ABSENT_BLOCK_CF};

#[derive(Clone)]
pub struct SyncDagStore {
    pub absent_dag_store: SyncAbsentBlockStore,
}

#[derive(Clone)]
pub struct SyncDagStoreConfig {
    pub cache_size: usize,
    pub rocksdb_config: RocksdbConfig,
}

impl Default for SyncDagStoreConfig {
    fn default() -> Self {
        Self {
            cache_size: 1,
            rocksdb_config: Default::default(),
        }
    }
}

impl SyncDagStoreConfig {
    pub fn new() -> Self {
        SyncDagStoreConfig::default()
    }

    pub fn create_with_params(cache_size: usize, rocksdb_config: RocksdbConfig) -> Self {
        Self {
            cache_size,
            rocksdb_config,
        }
    }
}

impl From<StorageConfig> for SyncDagStoreConfig {
    fn from(value: StorageConfig) -> Self {
        Self {
            cache_size: value.cache_size(),
            rocksdb_config: value.rocksdb_config(),
        }
    }
}

impl SyncDagStore {
    /// Creates or loads an existing storage from the provided directory path.
    pub fn create_from_path<P: AsRef<Path>>(
        db_path: P,
        config: SyncDagStoreConfig,
    ) -> anyhow::Result<Self> {
        let db = Arc::new(DBStorage::open_with_cfs(
            db_path,
            vec![SYNC_ABSENT_BLOCK_CF],
            false,
            config.rocksdb_config,
            None,
        )?);

        Ok(Self {
            absent_dag_store: SyncAbsentBlockStore::new(db, config.cache_size),
        })
    }

    pub fn create_for_testing() -> anyhow::Result<Self> {
        SyncDagStore::create_from_path(temp_dir(), SyncDagStoreConfig::default())
    }
}
