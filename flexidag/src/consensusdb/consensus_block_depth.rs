use std::sync::Arc;

use serde::{Deserialize, Serialize};

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
pub struct BlockDepthInfo {
    pub merge_depth_root: Hash,
    pub finality_point: Hash,
}

pub(crate) const DAG_BLOCK_DEPTH_INFO_STORE_CF: &str = "dag-block-depth-info-store";
define_schema!(
    BlockDepthInfoData,
    Hash,
    BlockDepthInfo,
    DAG_BLOCK_DEPTH_INFO_STORE_CF
);

impl KeyCodec<BlockDepthInfoData> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Self::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<BlockDepthInfoData> for BlockDepthInfo {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(&self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

pub trait BlockDepthInfoReader {
    fn get_block_depth_info(&self, hash: Hash) -> Result<BlockDepthInfo, StoreError>;
}

pub trait BlockDepthInfoStore: BlockDepthInfoReader {
    // This is append only
    fn insert(&self, hash: Hash, info: BlockDepthInfo) -> Result<(), StoreError>;
}

/// A DB + cache implementation of `DbBlockDepthInfoStore` trait, with concurrency support.
#[derive(Clone)]
pub struct DbBlockDepthInfoStore {
    db: Arc<DBStorage>,
    block_depth_info_access: CachedDbAccess<BlockDepthInfoData>,
}

impl DbBlockDepthInfoStore {
    pub fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            db: Arc::clone(&db),
            block_depth_info_access: CachedDbAccess::new(db.clone(), cache_size),
        }
    }
}

impl BlockDepthInfoReader for DbBlockDepthInfoStore {
    fn get_block_depth_info(&self, hash: Hash) -> Result<BlockDepthInfo, StoreError> {
        let result = self.block_depth_info_access.read(hash)?;
        Ok(result)
    }
}

impl BlockDepthInfoStore for DbBlockDepthInfoStore {
    fn insert(&self, hash: Hash, info: BlockDepthInfo) -> Result<(), StoreError> {
        self.block_depth_info_access
            .write(DirectDbWriter::new(&self.db), hash, info)?;
        Ok(())
    }
}
