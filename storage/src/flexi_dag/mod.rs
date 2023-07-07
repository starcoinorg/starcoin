use std::sync::Arc;

use crate::{
    accumulator::{AccumulatorStorage, DagBlockAccumulatorStorage},
    define_storage,
    storage::{CodecKVStore, StorageInstance, ValueCodec},
    SYNC_FLEXI_DAG_SNAPSHOT_PREFIX_NAME,
};
use anyhow::Result;
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, AccumulatorTreeStore};
use starcoin_crypto::HashValue;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncFlexiDagSnapshot {
    pub hashes: Vec<HashValue>,
    pub accumulator_info: AccumulatorInfo,
}

impl ValueCodec for SyncFlexiDagSnapshot {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

define_storage!(
    SyncFlexiDagSnapshotStorage,
    HashValue,
    SyncFlexiDagSnapshot,
    SYNC_FLEXI_DAG_SNAPSHOT_PREFIX_NAME
);

#[derive(Clone)]
pub struct SyncFlexiDagStorage {
    snapshot_storage: SyncFlexiDagSnapshotStorage,
    accumulator_storage: Arc<AccumulatorStorage<DagBlockAccumulatorStorage>>,
}

impl SyncFlexiDagStorage {
    pub fn new(instance: StorageInstance) -> Self {
        let snapshot_storage = SyncFlexiDagSnapshotStorage::new(instance.clone());
        let accumulator_storage = Arc::new(
            AccumulatorStorage::<DagBlockAccumulatorStorage>::new_dag_block_accumulator_storage(
                instance,
            ),
        );

        SyncFlexiDagStorage {
            snapshot_storage,
            accumulator_storage,
        }
    }

    pub fn get_accumulator_storage(&self) -> std::sync::Arc<dyn AccumulatorTreeStore> {
        self.accumulator_storage.clone()
    }

    pub fn put_hashes(&self, key: HashValue, accumulator_info: SyncFlexiDagSnapshot) -> Result<()> {
        self.snapshot_storage.put(key, accumulator_info)
    }

    pub fn get_hashes_by_hash(
        &self,
        hash: HashValue,
    ) -> std::result::Result<Option<SyncFlexiDagSnapshot>, anyhow::Error> {
        self.snapshot_storage.get(hash)
    }
}
