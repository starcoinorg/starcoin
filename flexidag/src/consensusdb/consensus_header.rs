use super::schema::{KeyCodec, ValueCodec};
use super::{
    db::DBStorage,
    error::{StoreError, StoreResult},
    prelude::CachedDbAccess,
    writer::{BatchDbWriter, DirectDbWriter},
};
use crate::define_schema;
use rocksdb::WriteBatch;
use starcoin_crypto::HashValue as Hash;
use starcoin_types::block::BlockHeader;
use starcoin_types::{
    blockhash::BlockLevel,
    consensus_header::{CompactHeaderData, HeaderWithBlockLevel},
    U256,
};
use std::sync::Arc;

pub trait HeaderStoreReader {
    fn get_daa_score(&self, hash: Hash) -> Result<u64, StoreError>;
    fn get_blue_score(&self, hash: Hash) -> Result<u64, StoreError>;
    fn get_difficulty(&self, hash: Hash) -> Result<U256, StoreError>;
}

pub trait HeaderStore: HeaderStoreReader {
    // This is append only
    fn insert(
        &self,
        hash: Hash,
        header: Arc<BlockHeader>,
        block_level: BlockLevel,
    ) -> Result<(), StoreError>;
}

pub(crate) const HEADERS_STORE_CF: &str = "headers-store";
pub(crate) const COMPACT_HEADER_DATA_STORE_CF: &str = "compact-header-data";

define_schema!(DagHeader, Hash, HeaderWithBlockLevel, HEADERS_STORE_CF);
define_schema!(
    CompactBlockHeader,
    Hash,
    CompactHeaderData,
    COMPACT_HEADER_DATA_STORE_CF
);

impl KeyCodec<DagHeader> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Hash::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<DagHeader> for HeaderWithBlockLevel {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(&self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl KeyCodec<CompactBlockHeader> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Hash::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<CompactBlockHeader> for CompactHeaderData {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(&self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

/// A DB + cache implementation of `HeaderStore` trait, with concurrency support.
#[derive(Clone)]
pub struct DbHeadersStore {
    db: Arc<DBStorage>,
    compact_headers_access: CachedDbAccess<CompactBlockHeader>,
}

impl DbHeadersStore {
    pub fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            db: Arc::clone(&db),
            compact_headers_access: CachedDbAccess::new(db, cache_size),
        }
    }

    pub fn clone_with_new_cache(&self, cache_size: usize) -> Self {
        Self::new(Arc::clone(&self.db), cache_size)
    }

    pub fn has(&self, hash: Hash) -> StoreResult<bool> {
        self.compact_headers_access.has(hash)
    }

    pub fn insert_batch(
        &self,
        batch: &mut WriteBatch,
        hash: Hash,
        header: Arc<BlockHeader>,
        _block_level: BlockLevel,
    ) -> Result<(), StoreError> {
        if self.compact_headers_access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.compact_headers_access.write(
            BatchDbWriter::new(batch),
            hash,
            CompactHeaderData {
                timestamp: header.timestamp(),
                difficulty: header.difficulty(),
            },
        )
    }
}

impl HeaderStoreReader for DbHeadersStore {
    fn get_daa_score(&self, _hash: Hash) -> Result<u64, StoreError> {
        unimplemented!()
    }

    fn get_blue_score(&self, _hash: Hash) -> Result<u64, StoreError> {
        unimplemented!()
    }

    fn get_difficulty(&self, hash: Hash) -> Result<U256, StoreError> {
        Ok(self.compact_headers_access.read(hash)?.difficulty)
    }
}

impl HeaderStore for DbHeadersStore {
    fn insert(
        &self,
        hash: Hash,
        header: Arc<BlockHeader>,
        _block_level: u8,
    ) -> Result<(), StoreError> {
        if self.compact_headers_access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.compact_headers_access.write(
            DirectDbWriter::new(&self.db),
            hash,
            CompactHeaderData {
                timestamp: header.timestamp(),
                difficulty: header.difficulty(),
            },
        )
    }
}
