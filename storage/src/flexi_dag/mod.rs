use std::{
    collections::BTreeSet,
    sync::Arc,
};

use crate::{
    accumulator::{AccumulatorStorage, DagBlockAccumulatorStorage},
    define_storage,
    storage::{CodecKVStore, StorageInstance, ValueCodec},
    SYNC_FLEXI_DAG_SNAPSHOT_PREFIX_NAME,
};
use anyhow::Result;
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::HashValue;
use starcoin_types::dag_block::KTotalDifficulty;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncFlexiDagSnapshot {
    pub child_hashes: Vec<HashValue>, // child nodes(tips), to get the relationship, use dag's relationship store
    pub accumulator_info: AccumulatorInfo,
    pub head_block_id: HashValue, // to initialize the BlockInfo
    pub k_total_difficulties: BTreeSet<KTotalDifficulty>, // the k-th smallest total difficulty
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncFlexiDagSnapshotHasher {
    pub child_hashes: Vec<HashValue>, // child nodes(tips), to get the relationship, use dag's relationship store
    pub head_block_id: HashValue, // to initialize the BlockInfo
    pub k_total_difficulties: BTreeSet<KTotalDifficulty>, // the k-th smallest total difficulty
}

impl SyncFlexiDagSnapshotHasher {
    pub fn to_snapshot(self, accumulator_info: AccumulatorInfo) -> SyncFlexiDagSnapshot {
        SyncFlexiDagSnapshot {
            child_hashes: self.child_hashes,
            accumulator_info,
            head_block_id: self.head_block_id,
            k_total_difficulties: self.k_total_difficulties,
        }
    }
}

impl From<SyncFlexiDagSnapshot> for SyncFlexiDagSnapshotHasher {
    fn from(mut value: SyncFlexiDagSnapshot) -> Self {
        value.child_hashes.sort();
        SyncFlexiDagSnapshotHasher { 
            child_hashes: value.child_hashes, 
            head_block_id: value.head_block_id, 
            k_total_difficulties: value.k_total_difficulties 
        }
    }
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
    HashValue, // accumulator leaf node
    SyncFlexiDagSnapshot,
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
