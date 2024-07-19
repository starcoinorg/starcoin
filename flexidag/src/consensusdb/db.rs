use super::{
    consenses_state::{DbDagStateStore, DAG_STATE_STORE_CF},
    error::StoreError,
    schemadb::{
        DbGhostdagStore, DbHeadersStore, DbReachabilityStore, DbRelationsStore, CHILDREN_CF,
        COMPACT_GHOST_DAG_STORE_CF, COMPACT_HEADER_DATA_STORE_CF, GHOST_DAG_STORE_CF,
        HEADERS_STORE_CF, PARENTS_CF, REACHABILITY_DATA_CF,
    },
};
use crate::consensusdb::error::StoreError::DBIoError;
use crate::consensusdb::schemadb::{REACHABILITY_CHILDREN_DATA_CF, REACHABILITY_FCS_DATA_CF};
use parking_lot::RwLock;
use rocksdb::{WriteBatch, WriteOptions};
use starcoin_config::{RocksdbConfig, StorageConfig};
pub(crate) use starcoin_storage::db_storage::DBStorage;
use std::{path::Path, sync::Arc};

#[derive(Clone)]
pub struct FlexiDagStorage {
    db: Arc<DBStorage>,
    pub ghost_dag_store: DbGhostdagStore,
    pub header_store: DbHeadersStore,
    pub reachability_store: Arc<RwLock<DbReachabilityStore>>,
    pub relations_store: Arc<RwLock<DbRelationsStore>>,
    pub state_store: Arc<RwLock<DbDagStateStore>>,
}

#[derive(Clone)]
pub struct FlexiDagStorageConfig {
    pub cache_size: usize,
    pub rocksdb_config: RocksdbConfig,
}
impl Default for FlexiDagStorageConfig {
    fn default() -> Self {
        Self {
            cache_size: 1,
            rocksdb_config: Default::default(),
        }
    }
}
impl FlexiDagStorageConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_with_params(cache_size: usize, rocksdb_config: RocksdbConfig) -> Self {
        Self {
            cache_size,
            rocksdb_config,
        }
    }
}

impl From<StorageConfig> for FlexiDagStorageConfig {
    fn from(value: StorageConfig) -> Self {
        Self {
            cache_size: value.cache_size(),
            rocksdb_config: value.rocksdb_config(),
        }
    }
}

impl FlexiDagStorage {
    /// Creates or loads an existing storage from the provided directory path.
    pub fn create_from_path<P: AsRef<Path>>(
        db_path: P,
        config: FlexiDagStorageConfig,
    ) -> Result<Self, StoreError> {
        let db = Arc::new(
            DBStorage::open_with_cfs(
                db_path,
                vec![
                    // consensus headers
                    HEADERS_STORE_CF,
                    COMPACT_HEADER_DATA_STORE_CF,
                    // consensus relations
                    PARENTS_CF,
                    CHILDREN_CF,
                    // consensus reachability
                    REACHABILITY_DATA_CF,
                    REACHABILITY_CHILDREN_DATA_CF,
                    REACHABILITY_FCS_DATA_CF,
                    // consensus ghostdag
                    GHOST_DAG_STORE_CF,
                    COMPACT_GHOST_DAG_STORE_CF,
                    DAG_STATE_STORE_CF,
                ],
                false,
                config.rocksdb_config,
                None,
            )
            .map_err(|e| StoreError::DBIoError(e.to_string()))?,
        );

        Ok(Self {
            db: db.clone(),
            ghost_dag_store: DbGhostdagStore::new(db.clone(), 1, config.cache_size),

            header_store: DbHeadersStore::new(db.clone(), config.cache_size),
            reachability_store: Arc::new(RwLock::new(DbReachabilityStore::new(
                db.clone(),
                config.cache_size,
            ))),
            relations_store: Arc::new(RwLock::new(DbRelationsStore::new(
                db.clone(),
                1,
                config.cache_size,
            ))),
            state_store: Arc::new(RwLock::new(DbDagStateStore::new(db, config.cache_size))),
        })
    }

    pub fn write_batch(&self, batch: WriteBatch) -> Result<(), StoreError> {
        self.write_batch_opt(batch, &WriteOptions::default())
    }

    pub fn write_batch_opt(
        &self,
        batch: WriteBatch,
        opts: &WriteOptions,
    ) -> Result<(), StoreError> {
        self.db
            .raw_write_batch_opt(batch, opts)
            .map_err(|e| DBIoError(e.to_string()))
    }

    pub fn flush_opt(&self, wait: bool) -> Result<(), StoreError> {
        self.db
            .flush_opt(wait)
            .map_err(|e| DBIoError(e.to_string()))?;
        Ok(())
    }
}
