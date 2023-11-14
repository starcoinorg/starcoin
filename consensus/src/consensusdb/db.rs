use super::{
    error::StoreError,
    schemadb::{
        DbGhostdagStore, DbHeadersStore, DbReachabilityStore, DbRelationsStore, CHILDREN_CF,
        COMPACT_GHOST_DAG_STORE_CF, COMPACT_HEADER_DATA_STORE_CF, GHOST_DAG_STORE_CF,
        HEADERS_STORE_CF, PARENTS_CF, REACHABILITY_DATA_CF,
    },
};
use starcoin_config::{RocksdbConfig, StorageConfig};
pub(crate) use starcoin_storage::db_storage::DBStorage;
use std::{path::Path, sync::Arc};

#[derive(Clone)]
pub struct FlexiDagStorage {
    pub ghost_dag_store: DbGhostdagStore,
    pub header_store: DbHeadersStore,
    pub reachability_store: DbReachabilityStore,
    pub relations_store: DbRelationsStore,
}

#[derive(Clone, Default)]
pub struct FlexiDagStorageConfig {
    pub cache_size: usize,
    pub rocksdb_config: RocksdbConfig,
}

impl FlexiDagStorageConfig {
    pub fn new() -> Self {
        FlexiDagStorageConfig::default()
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
                    // consensus ghostdag
                    GHOST_DAG_STORE_CF,
                    COMPACT_GHOST_DAG_STORE_CF,
                ],
                false,
                config.rocksdb_config,
                None,
            )
            .map_err(|e| StoreError::DBIoError(e.to_string()))?,
        );

        Ok(Self {
            ghost_dag_store: DbGhostdagStore::new(db.clone(), 1, config.cache_size),

            header_store: DbHeadersStore::new(db.clone(), config.cache_size),
            reachability_store: DbReachabilityStore::new(db.clone(), config.cache_size),
            relations_store: DbRelationsStore::new(db, 1, config.cache_size),
        })
    }
}
