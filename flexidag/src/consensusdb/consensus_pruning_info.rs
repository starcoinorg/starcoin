use std::sync::Arc;

use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::error;

use crate::define_schema;
use starcoin_crypto::HashValue as Hash;
use starcoin_storage::db_storage::DBStorage;

use super::{
    access::CachedDbAccess,
    error::StoreError,
    schema::{KeyCodec, ValueCodec},
    writer::DirectDbWriter,
};

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug, Default)]
pub struct PruningPointInfo {
    pruning_point: Hash,
}

pub(crate) const PRUNING_POINT_INFO_STORE_CF: &str = "pruning_point_info_store_cf";
define_schema!(
    PruningPointInfoData,
    Hash,
    PruningPointInfo,
    PRUNING_POINT_INFO_STORE_CF
);

impl KeyCodec<PruningPointInfoData> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Self::from_slice(data).map_err(|e| {
            StoreError::DecodeError(format!(
                "failed to decode the key for PruningPointInfoData for error: {}",
                e
            ))
        })
    }
}
impl ValueCodec<PruningPointInfoData> for PruningPointInfo {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(&self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

pub trait PruningPointInfoReader {
    fn get_pruning_point_info(&self) -> Result<Option<PruningPointInfo>, StoreError>;
}

pub trait PruningPointInfoWriter: PruningPointInfoReader {
    fn insert(&self, info: PruningPointInfo) -> Result<(), StoreError>;
}

/// A DB + cache implementation of `PruningPointInfoStore` trait, with concurrency support.
#[derive(Clone)]
pub struct PruningPointInfoStore {
    db: Arc<DBStorage>,
    pruning_point_info_access: CachedDbAccess<PruningPointInfoData>,
}

impl PruningPointInfoStore {
    pub fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            db: Arc::clone(&db),
            pruning_point_info_access: CachedDbAccess::new(db.clone(), cache_size),
        }
    }
}

impl PruningPointInfoReader for PruningPointInfoStore {
    fn get_pruning_point_info(&self) -> Result<Option<PruningPointInfo>, StoreError> {
        let result = match self.pruning_point_info_access.read(Hash::zero()) {
            Ok(info) => Some(info),
            Err(e) => match e {
                StoreError::KeyNotFound(_) => None,
                _ => {
                    error!("get_pruning_point_info error: {:?} for id: {:?}, the candidate in tips referring too many red blocks will not be filtered.", e, Hash::zero());
                    None
                }
            },
        };
        Ok(result)
    }
}

impl PruningPointInfoWriter for PruningPointInfoStore {
    fn insert(&self, info: PruningPointInfo) -> Result<(), StoreError> {
        self.pruning_point_info_access
            .write(DirectDbWriter::new(&self.db), Hash::zero(), info)?;
        Ok(())
    }
}
