use std::sync::Arc;

use crate::{
    accumulator::{AccumulatorStorage, DagBlockAccumulatorStorage},
    define_storage,
    storage::{StorageInstance, ValueCodec, CodecKVStore},
    SYNC_FLEXI_DAG_SNAPSHOT_PREFIX_NAME,
};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockInfoExt;

impl ValueCodec for BlockInfoExt {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

define_storage!(
    SyncFlexiDagSnapshotStorage,
    HashValue, // accumulator leaf node
    BlockInfoExt,
    SYNC_FLEXI_DAG_SNAPSHOT_PREFIX_NAME
);

#[derive(Clone)]
pub struct SyncFlexiDagStorage {
    snapshot_storage: Arc<SyncFlexiDagSnapshotStorage>,
    accumulator_storage: AccumulatorStorage<DagBlockAccumulatorStorage>,
}

impl SyncFlexiDagStorage {
    pub fn new(instance: StorageInstance) -> Self {
        let snapshot_storage = Arc::new(SyncFlexiDagSnapshotStorage::new(instance.clone()));
        let accumulator_storage =
            AccumulatorStorage::<DagBlockAccumulatorStorage>::new_dag_block_accumulator_storage(
                instance,
            );

        SyncFlexiDagStorage {
            snapshot_storage,
            accumulator_storage,
        }
    }

    pub fn get_accumulator_storage(&self) -> AccumulatorStorage<DagBlockAccumulatorStorage> {
        self.accumulator_storage.clone()
    }

    pub fn get_snapshot_storage(&self) -> Arc<SyncFlexiDagSnapshotStorage> {
        self.snapshot_storage.clone()
    }

    pub fn put_hashes(&self, key: HashValue, block_info_ext: BlockInfoExt) -> Result<()> {
        self.snapshot_storage.put(key, block_info_ext)
    }

    pub fn get_hashes_by_hash(
        &self,
        hash: HashValue,
    ) -> std::result::Result<Option<BlockInfoExt>, anyhow::Error> {
        self.snapshot_storage.get(hash)
    }
}
