use super::schema::{KeyCodec, ValueCodec};
use super::{db::DBStorage, error::StoreError, prelude::CachedDbAccess, writer::DirectDbWriter};
use crate::define_schema;
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue as Hash;
use std::sync::Arc;

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug, Default)]
pub struct DagState {
    pub tips: Vec<Hash>,
}

pub(crate) const DAG_STATE_STORE_CF: &str = "dag-state-store";
define_schema!(DagStateData, Hash, DagState, DAG_STATE_STORE_CF);

impl KeyCodec<DagStateData> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Self::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<DagStateData> for DagState {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(&self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

pub trait DagStateReader {
    fn get_state_by_hash(&self, hash: Hash) -> Result<DagState, StoreError>;
}

pub trait DagStateStore: DagStateReader {
    // This is append only
    fn insert(&self, hash: Hash, state: DagState) -> Result<(), StoreError>;
}

/// A DB + cache implementation of `HeaderStore` trait, with concurrency support.
#[derive(Clone)]
pub struct DbDagStateStore {
    db: Arc<DBStorage>,
    dag_state_access: CachedDbAccess<DagStateData>,
}

impl DbDagStateStore {
    pub fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            db: Arc::clone(&db),
            dag_state_access: CachedDbAccess::new(db.clone(), cache_size),
        }
    }
}

impl DagStateReader for DbDagStateStore {
    fn get_state_by_hash(&self, hash: Hash) -> Result<DagState, StoreError> {
        let result = self.dag_state_access.read(hash)?;
        Ok(result)
    }
}

impl DagStateStore for DbDagStateStore {
    fn insert(&self, hash: Hash, state: DagState) -> Result<(), StoreError> {
        self.dag_state_access
            .write(DirectDbWriter::new(&self.db), hash, state)?;
        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct DagStateView {
    pub tips: Vec<Hash>,
    pub pruning_point: Hash,
}

impl DagStateView {
    pub fn into_state(self) -> DagState {
        DagState { tips: self.tips }
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ReachabilityView {
    pub ancestor: Hash,
    pub descendants: Vec<Hash>,
}
