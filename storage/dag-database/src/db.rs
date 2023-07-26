use crate::consensus::{
    DbGhostdagStore, DbHeadersStore, DbReachabilityStore, DbRelationsStore, CHILDREN_CF,
    COMPACT_GHOST_DAG_STORE_CF, COMPACT_HEADER_DATA_STORE_CF, GHOST_DAG_STORE_CF, HEADERS_STORE_CF,
    PARENTS_CF, REACHABILITY_DATA_CF,
};
use crate::errors::StoreError;
use starcoin_config::RocksdbConfig;
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
pub struct GhostDagStoreConfig {
    pub block_level: u8,
    pub cache_size: u64,
}

#[derive(Clone, Default)]
pub struct HeaderStoreConfig {
    pub cache_size: u64,
}

#[derive(Clone, Default)]
pub struct ReachabilityStoreConfig {
    pub cache_size: u64,
}

#[derive(Clone, Default)]
pub struct RelationsStoreConfig {
    pub block_level: u8,
    pub cache_size: u64,
}

#[derive(Clone, Default)]
pub struct FlexiDagStorageConfig {
    pub parallelism: u64,
    pub gds_conf: GhostDagStoreConfig,
    pub hs_conf: HeaderStoreConfig,
    pub rbs_conf: ReachabilityStoreConfig,
    pub rs_conf: RelationsStoreConfig,
}

impl FlexiDagStorageConfig {
    pub fn new() -> Self {
        FlexiDagStorageConfig::default()
    }

    pub fn create_with_params(parallelism: u64, block_level: u8, cache_size: u64) -> Self {
        Self {
            parallelism,
            gds_conf: GhostDagStoreConfig {
                block_level,
                cache_size,
            },
            hs_conf: HeaderStoreConfig { cache_size },
            rbs_conf: ReachabilityStoreConfig { cache_size },
            rs_conf: RelationsStoreConfig {
                block_level,
                cache_size,
            },
        }
    }

    pub fn update_parallelism(mut self, parallelism: u64) -> Self {
        self.parallelism = parallelism;
        self
    }

    pub fn update_ghost_dag_conf(mut self, gds_conf: GhostDagStoreConfig) -> Self {
        self.gds_conf = gds_conf;
        self
    }

    pub fn update_headers_conf(mut self, hs_conf: HeaderStoreConfig) -> Self {
        self.hs_conf = hs_conf;
        self
    }

    pub fn update_reachability_conf(mut self, rbs_conf: ReachabilityStoreConfig) -> Self {
        self.rbs_conf = rbs_conf;
        self
    }

    pub fn update_relations_conf(mut self, rs_conf: RelationsStoreConfig) -> Self {
        self.rs_conf = rs_conf;
        self
    }
}

impl FlexiDagStorage {
    /// Creates or loads an existing storage from the provided directory path.
    pub fn create_from_path<P: AsRef<Path>>(
        db_path: P,
        config: FlexiDagStorageConfig,
    ) -> Result<Self, StoreError> {
        let rocksdb_config = RocksdbConfig {
            parallelism: config.parallelism,
            ..Default::default()
        };

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
                rocksdb_config,
                None,
            )
            .map_err(|e| StoreError::DBIoError(e.to_string()))?,
        );

        Ok(Self {
            ghost_dag_store: DbGhostdagStore::new(
                db.clone(),
                config.gds_conf.block_level,
                config.gds_conf.cache_size,
            ),

            header_store: DbHeadersStore::new(db.clone(), config.hs_conf.cache_size),
            reachability_store: DbReachabilityStore::new(db.clone(), config.rbs_conf.cache_size),
            relations_store: DbRelationsStore::new(
                db,
                config.rs_conf.block_level,
                config.rs_conf.cache_size,
            ),
        })
    }
}
