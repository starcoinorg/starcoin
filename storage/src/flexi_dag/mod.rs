use std::sync::Arc;

use crate::{
    accumulator::{AccumulatorStorage, DagBlockAccumulatorStorage},
    define_storage,
    storage::{CodecKVStore, StorageInstance, ValueCodec},
    DAG_TIPS_PREFIX_NAME, SYNC_FLEXI_DAG_SNAPSHOT_PREFIX_NAME,
};
use anyhow::Result;
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DagTips {
    pub tips: Vec<HashValue>,
}

impl ValueCodec for DagTips {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncFlexiDagSnapshot {
    pub dag_blocks: Vec<HashValue>,
    pub number: BlockNumber,
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

define_storage!(DagTipsStorage, HashValue, DagTips, DAG_TIPS_PREFIX_NAME);

#[derive(Clone)]
pub struct SyncFlexiDagStorage {
    snapshot_storage: Arc<SyncFlexiDagSnapshotStorage>,
    dag_tips_storage: Arc<DagTipsStorage>,
    accumulator_storage: AccumulatorStorage<DagBlockAccumulatorStorage>,
}

impl SyncFlexiDagStorage {
    pub fn new(instance: StorageInstance) -> Self {
        let snapshot_storage = Arc::new(SyncFlexiDagSnapshotStorage::new(instance.clone()));
        let dag_tips_storage = Arc::new(DagTipsStorage::new(instance.clone()));
        let accumulator_storage =
            AccumulatorStorage::<DagBlockAccumulatorStorage>::new_dag_block_accumulator_storage(
                instance,
            );

        SyncFlexiDagStorage {
            snapshot_storage,
            dag_tips_storage,
            accumulator_storage,
        }
    }

    pub fn get_accumulator_storage(&self) -> AccumulatorStorage<DagBlockAccumulatorStorage> {
        self.accumulator_storage.clone()
    }

    pub fn get_snapshot_storage(&self) -> Arc<SyncFlexiDagSnapshotStorage> {
        self.snapshot_storage.clone()
    }

    pub fn get_dag_tips_storage(&self) -> Arc<DagTipsStorage> {
        self.dag_tips_storage.clone()
    }

    pub fn put_hashes(&self, key: HashValue, accumulator_info: SyncFlexiDagSnapshot) -> Result<()> {
        self.snapshot_storage.put(key, accumulator_info)
    }

    pub fn get_dag_tips(&self) -> std::result::Result<Option<DagTips>, anyhow::Error> {
        self.dag_tips_storage.get(0.into())
    }

    pub fn save_dag_tips(&self, tips: Vec<HashValue>) -> Result<()> {
        self.dag_tips_storage.put(0.into(), DagTips { tips })
    }

    pub fn get_hashes_by_hash(
        &self,
        hash: HashValue,
    ) -> std::result::Result<Option<SyncFlexiDagSnapshot>, anyhow::Error> {
        self.snapshot_storage.get(hash)
    }
}
