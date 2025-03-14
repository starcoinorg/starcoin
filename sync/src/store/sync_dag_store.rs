use std::{path::Path, sync::Arc};

use anyhow::format_err;
use starcoin_config::{temp_dir, RocksdbConfig, StorageConfig};
use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::{prelude::StoreError, schemadb::REACHABILITY_DATA_CF};
use starcoin_logger::prelude::error;
use starcoin_storage::db_storage::{DBStorage, SchemaIterator};
use starcoin_types::block::{Block, BlockNumber};

use super::sync_absent_ancestor::{
    AbsentDagBlockStoreReader, AbsentDagBlockStoreWriter, DagSyncBlock, SyncAbsentBlockStore,
    SYNC_ABSENT_BLOCK_CF,
};

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
        Self::default()
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
        let db = Arc::new(
            DBStorage::open_with_cfs(
                db_path,
                vec![SYNC_ABSENT_BLOCK_CF, REACHABILITY_DATA_CF],
                false,
                config.rocksdb_config,
                None,
            )
            .map_err(|e| format_err!("Failed to open database: {:?}", e))?,
        );

        Ok(Self {
            absent_dag_store: SyncAbsentBlockStore::new(db.clone(), config.cache_size),
        })
    }

    pub fn create_for_testing() -> anyhow::Result<Self> {
        Self::create_from_path(temp_dir(), SyncDagStoreConfig::default())
    }

    pub fn save_block(&self, block: Block) -> anyhow::Result<()> {
        match self
            .absent_dag_store
            .get_absent_block_by_id(block.header().number(), block.id())
        {
            Ok(sync_dag_block) => {
                if sync_dag_block
                    .block
                    .ok_or_else(|| {
                        format_err!(
                            "The sync dag block:{:?} is in sync dag block store but block is None.",
                            block.id()
                        )
                    })?
                    .header()
                    .id()
                    == block.id()
                {
                    Ok(())
                } else {
                    Err(format_err!("The sync dag block:{:?} is in sync dag block store but block is not equal.", block.id()))
                }
            }
            Err(e) => match e {
                StoreError::KeyNotFound(_) => {
                    self.absent_dag_store
                        .save_absent_block(vec![DagSyncBlock {
                            block: Some(block.clone()),
                        }])
                        .map_err(|e| format_err!("Failed to save absent block: {:?}", e))?;
                    Ok(())
                }
                _ => Err(format_err!(
                    "Failed to save block:{:?} into sync dag store. db error: {:?}",
                    block.id(),
                    e
                )),
            },
        }
    }

    pub fn iter_at_first(&self) -> anyhow::Result<SchemaIterator<Vec<u8>, Vec<u8>>> {
        self.absent_dag_store.iter_at_first()
    }

    pub fn delete_dag_sync_block(
        &self,
        number: BlockNumber,
        block_id: HashValue,
    ) -> anyhow::Result<()> {
        self.absent_dag_store.delete_absent_block(number, block_id)
    }

    pub fn get_dag_sync_block(
        &self,
        number: BlockNumber,
        block_id: HashValue,
    ) -> anyhow::Result<DagSyncBlock, StoreError> {
        self.absent_dag_store
            .get_absent_block_by_id(number, block_id)
            .map_err(|e| {
                error!(
                    "Failed to get DAG sync block with number: {}, block_id: {}. Error: {:?}",
                    number, block_id, e
                );
                e
            })
    }

    pub(crate) fn delete_all_dag_sync_block(&self) -> anyhow::Result<()> {
        self.absent_dag_store.delete_all_absent_block()
    }
}
