use std::{collections::BTreeMap, path::Path, sync::Arc};

use anyhow::format_err;
use starcoin_config::{temp_dir, RocksdbConfig, StorageConfig};
use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::schema::ValueCodec;
use starcoin_dag::consensusdb::{prelude::StoreError, schemadb::REACHABILITY_DATA_CF};
use starcoin_logger::prelude::{error, info, warn};
use starcoin_storage::db_storage::{DBStorage, SchemaIterator};
use starcoin_types::block::{Block, BlockNumber};

use super::sync_absent_ancestor::{
    AbsentDagBlockStoreReader, AbsentDagBlockStoreWriter, DagSyncBlock, DagSyncBlockKey,
    SyncAbsentBlockStore, SYNC_ABSENT_BLOCK_CF,
};

#[derive(Clone)]
pub struct SyncDagStore {
    pub absent_dag_store: SyncAbsentBlockStore,
    memory_limit_bytes: usize,
    memory_state: Arc<std::sync::Mutex<InMemoryDagStore>>,
}

#[derive(Clone)]
pub struct SyncDagStoreConfig {
    pub cache_size: usize,
    pub rocksdb_config: RocksdbConfig,
    pub memory_limit_bytes: usize,
}

#[derive(Clone)]
struct InMemoryDagEntry {
    block: DagSyncBlock,
    encoded_size: usize,
}

#[derive(Default)]
struct InMemoryDagStore {
    blocks: BTreeMap<DagSyncBlockKey, InMemoryDagEntry>,
    total_bytes: usize,
    spilled_to_disk: bool,
}

impl Default for SyncDagStoreConfig {
    fn default() -> Self {
        Self {
            cache_size: 1,
            rocksdb_config: Default::default(),
            memory_limit_bytes: 1024 * 1024 * 1024,
        }
    }
}

impl SyncDagStoreConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_with_params(
        cache_size: usize,
        rocksdb_config: RocksdbConfig,
        memory_limit_bytes: usize,
    ) -> Self {
        Self {
            cache_size,
            rocksdb_config,
            memory_limit_bytes,
        }
    }
}

impl From<StorageConfig> for SyncDagStoreConfig {
    fn from(value: StorageConfig) -> Self {
        Self {
            cache_size: value.cache_size(),
            rocksdb_config: value.rocksdb_config(),
            memory_limit_bytes: SyncDagStoreConfig::default().memory_limit_bytes,
        }
    }
}

impl SyncDagStore {
    fn save_block_to_disk(&self, block: Block) -> anyhow::Result<()> {
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
                    Err(format_err!(
                        "The sync dag block:{:?} is in sync dag block store but block is not equal.",
                        block.id()
                    ))
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

    fn spill_memory_to_disk(&self, state: &mut InMemoryDagStore) -> anyhow::Result<()> {
        if state.blocks.is_empty() {
            state.spilled_to_disk = true;
            return Ok(());
        }

        let blocks = state
            .blocks
            .values()
            .map(|entry| entry.block.clone())
            .collect::<Vec<_>>();
        self.absent_dag_store
            .save_absent_block(blocks)
            .map_err(|e| format_err!("Failed to spill in-memory dag blocks to disk: {:?}", e))?;

        info!("sync dag store switched to disk mode");
        state.blocks.clear();
        state.total_bytes = 0;
        state.spilled_to_disk = true;
        Ok(())
    }

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
        let absent_dag_store = SyncAbsentBlockStore::new(db.clone(), config.cache_size);
        let has_existing_blocks_on_disk = {
            let mut iter = absent_dag_store
                .iter_at_first()
                .map_err(|e| format_err!("Failed to inspect sync dag store iterator: {:?}", e))?;
            iter.next().transpose()?.is_some()
        };
        let mut memory_store = InMemoryDagStore::default();
        if has_existing_blocks_on_disk {
            info!("sync dag store found existing disk data, starting in disk mode");
            memory_store.spilled_to_disk = true;
        }

        Ok(Self {
            absent_dag_store,
            memory_limit_bytes: config.memory_limit_bytes,
            memory_state: Arc::new(std::sync::Mutex::new(memory_store)),
        })
    }

    pub fn create_for_testing() -> anyhow::Result<Self> {
        Self::create_from_path(temp_dir(), SyncDagStoreConfig::default())
    }

    pub fn save_block(&self, block: Block) -> anyhow::Result<()> {
        let key = DagSyncBlockKey {
            number: block.header().number(),
            block_id: block.id(),
        };
        let sync_block = DagSyncBlock {
            block: Some(block.clone()),
        };
        let encoded_size = bcs_ext::to_bytes(&sync_block)
            .map_err(|e| format_err!("Failed to encode dag sync block for memory sizing: {:?}", e))?
            .len();

        let mut memory_state = self
            .memory_state
            .lock()
            .map_err(|_| format_err!("sync dag store memory mutex poisoned"))?;

        if !memory_state.spilled_to_disk {
            if let Some(entry) = memory_state.blocks.get(&key) {
                let existing_block = entry.block.block.as_ref().ok_or_else(|| {
                    format_err!(
                        "The sync dag block:{:?} is in memory sync dag store but block is None.",
                        block.id()
                    )
                })?;
                if existing_block.id() == block.id() {
                    return Ok(());
                } else {
                    return Err(format_err!(
                        "The sync dag block:{:?} is in memory sync dag store but block is not equal.",
                        block.id()
                    ));
                }
            }

            if self.memory_limit_bytes > 0
                && memory_state.total_bytes.saturating_add(encoded_size) > self.memory_limit_bytes
            {
                warn!(
                    "sync dag store memory threshold exceeded ({} > {}), spilling to disk",
                    memory_state.total_bytes.saturating_add(encoded_size),
                    self.memory_limit_bytes
                );
                self.spill_memory_to_disk(&mut memory_state)?;
            } else {
                memory_state.blocks.insert(
                    key,
                    InMemoryDagEntry {
                        block: sync_block,
                        encoded_size,
                    },
                );
                memory_state.total_bytes = memory_state.total_bytes.saturating_add(encoded_size);
                return Ok(());
            }
        }

        drop(memory_state);
        self.save_block_to_disk(block)
    }

    pub fn all_blocks(&self) -> anyhow::Result<Vec<Block>> {
        let memory_state = self
            .memory_state
            .lock()
            .map_err(|_| format_err!("sync dag store memory mutex poisoned"))?;
        if !memory_state.spilled_to_disk {
            return memory_state
                .blocks
                .values()
                .map(|entry| {
                    entry.block.block.clone().ok_or_else(|| {
                        format_err!("block in sync dag block should not be none in memory")
                    })
                })
                .collect();
        }
        drop(memory_state);

        let iter = self.absent_dag_store.iter_at_first()?;
        let mut blocks = Vec::new();
        for result in iter {
            let (_, data_raw) = result?;
            let dag_sync_block = DagSyncBlock::decode_value(&data_raw)?;
            let block = dag_sync_block
                .block
                .ok_or_else(|| format_err!("block in sync dag block should not be none"))?;
            blocks.push(block);
        }
        Ok(blocks)
    }

    #[allow(dead_code)]
    pub fn iter_at_first(&self) -> anyhow::Result<SchemaIterator<Vec<u8>, Vec<u8>>> {
        self.absent_dag_store.iter_at_first()
    }

    pub fn delete_dag_sync_block(
        &self,
        number: BlockNumber,
        block_id: HashValue,
    ) -> anyhow::Result<()> {
        let key = DagSyncBlockKey { number, block_id };
        let mut memory_state = self
            .memory_state
            .lock()
            .map_err(|_| format_err!("sync dag store memory mutex poisoned"))?;
        if !memory_state.spilled_to_disk {
            if let Some(entry) = memory_state.blocks.remove(&key) {
                memory_state.total_bytes =
                    memory_state.total_bytes.saturating_sub(entry.encoded_size);
                return Ok(());
            }
            return Err(format_err!(
                "failed to delete absent block from memory: {}",
                block_id
            ));
        }
        drop(memory_state);
        self.absent_dag_store.delete_absent_block(number, block_id)
    }

    pub fn get_dag_sync_block(
        &self,
        number: BlockNumber,
        block_id: HashValue,
    ) -> anyhow::Result<DagSyncBlock, StoreError> {
        let key = DagSyncBlockKey { number, block_id };
        let memory_state = self.memory_state.lock().map_err(|_| {
            StoreError::DBIoError("sync dag store memory mutex poisoned".to_string())
        })?;
        if !memory_state.spilled_to_disk {
            return memory_state
                .blocks
                .get(&key)
                .map(|entry| entry.block.clone())
                .ok_or_else(|| StoreError::KeyNotFound(format!("{:?}", key)));
        }
        drop(memory_state);

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
        let mut memory_state = self
            .memory_state
            .lock()
            .map_err(|_| format_err!("sync dag store memory mutex poisoned"))?;
        memory_state.blocks.clear();
        memory_state.total_bytes = 0;
        memory_state.spilled_to_disk = false;
        drop(memory_state);
        self.absent_dag_store.delete_all_absent_block()
    }

    #[cfg(test)]
    pub fn spilled_to_disk(&self) -> bool {
        self.memory_state
            .lock()
            .map(|state| state.spilled_to_disk)
            .unwrap_or(true)
    }
}
