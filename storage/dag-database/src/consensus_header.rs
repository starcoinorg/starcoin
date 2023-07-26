use crate::{
    db::DBStorage,
    errors::{StoreError, StoreResult},
    prelude::CachedDbAccess,
    writer::{BatchDbWriter, DirectDbWriter},
};
use rocksdb::WriteBatch;
use starcoin_crypto::HashValue as Hash;
use starcoin_types::U256;
use starcoin_types::{
    blockhash::BlockLevel,
    header::{CompactHeaderData, ConsensusHeader, Header, HeaderWithBlockLevel},
};
use std::sync::Arc;

pub trait HeaderStoreReader {
    fn get_daa_score(&self, hash: Hash) -> Result<u64, StoreError>;
    fn get_blue_score(&self, hash: Hash) -> Result<u64, StoreError>;
    fn get_timestamp(&self, hash: Hash) -> Result<u64, StoreError>;
    fn get_difficulty(&self, hash: Hash) -> Result<U256, StoreError>;
    fn get_header(&self, hash: Hash) -> Result<Arc<Header>, StoreError>;
    fn get_header_with_block_level(&self, hash: Hash) -> Result<HeaderWithBlockLevel, StoreError>;
    fn get_compact_header_data(&self, hash: Hash) -> Result<CompactHeaderData, StoreError>;
}

pub trait HeaderStore: HeaderStoreReader {
    // This is append only
    fn insert(
        &self,
        hash: Hash,
        header: Arc<Header>,
        block_level: BlockLevel,
    ) -> Result<(), StoreError>;
}

pub(crate) const HEADERS_STORE_CF: &str = "headers-store";
pub(crate) const COMPACT_HEADER_DATA_STORE_CF: &str = "compact-header-data";

/// A DB + cache implementation of `HeaderStore` trait, with concurrency support.
#[derive(Clone)]
pub struct DbHeadersStore {
    db: Arc<DBStorage>,
    compact_headers_access: CachedDbAccess<Hash, CompactHeaderData>,
    headers_access: CachedDbAccess<Hash, HeaderWithBlockLevel>,
}

impl DbHeadersStore {
    pub fn new(db: Arc<DBStorage>, cache_size: u64) -> Self {
        Self {
            db: Arc::clone(&db),
            compact_headers_access: CachedDbAccess::new(
                Arc::clone(&db),
                cache_size,
                COMPACT_HEADER_DATA_STORE_CF,
            ),
            headers_access: CachedDbAccess::new(db, cache_size, HEADERS_STORE_CF),
        }
    }

    pub fn clone_with_new_cache(&self, cache_size: u64) -> Self {
        Self::new(Arc::clone(&self.db), cache_size)
    }

    pub fn has(&self, hash: Hash) -> StoreResult<bool> {
        self.headers_access.has(hash)
    }

    pub fn get_header(&self, hash: Hash) -> Result<Header, StoreError> {
        let result = self.headers_access.read(hash)?;
        Ok((*result.header).clone())
    }

    pub fn insert_batch(
        &self,
        batch: &mut WriteBatch,
        hash: Hash,
        header: Arc<Header>,
        block_level: BlockLevel,
    ) -> Result<(), StoreError> {
        if self.headers_access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.headers_access.write(
            BatchDbWriter::new(batch),
            hash,
            HeaderWithBlockLevel {
                header: header.clone(),
                block_level,
            },
        )?;
        self.compact_headers_access.write(
            BatchDbWriter::new(batch),
            hash,
            CompactHeaderData {
                timestamp: header.timestamp(),
                difficulty: header.difficulty(),
            },
        )?;
        Ok(())
    }
}

impl HeaderStoreReader for DbHeadersStore {
    fn get_daa_score(&self, _hash: Hash) -> Result<u64, StoreError> {
        unimplemented!()
    }

    fn get_blue_score(&self, _hash: Hash) -> Result<u64, StoreError> {
        unimplemented!()
    }

    fn get_timestamp(&self, hash: Hash) -> Result<u64, StoreError> {
        if let Some(header_with_block_level) = self.headers_access.read_from_cache(hash)? {
            return Ok(header_with_block_level.header.timestamp());
        }
        Ok(self.compact_headers_access.read(hash)?.timestamp)
    }

    fn get_difficulty(&self, hash: Hash) -> Result<U256, StoreError> {
        if let Some(header_with_block_level) = self.headers_access.read_from_cache(hash)? {
            return Ok(header_with_block_level.header.difficulty());
        }
        Ok(self.compact_headers_access.read(hash)?.difficulty)
    }

    fn get_header(&self, hash: Hash) -> Result<Arc<Header>, StoreError> {
        Ok(self.headers_access.read(hash)?.header)
    }

    fn get_header_with_block_level(&self, hash: Hash) -> Result<HeaderWithBlockLevel, StoreError> {
        self.headers_access.read(hash)
    }

    fn get_compact_header_data(&self, hash: Hash) -> Result<CompactHeaderData, StoreError> {
        if let Some(header_with_block_level) = self.headers_access.read_from_cache(hash)? {
            return Ok(CompactHeaderData {
                timestamp: header_with_block_level.header.timestamp(),
                difficulty: header_with_block_level.header.difficulty(),
            });
        }
        self.compact_headers_access.read(hash)
    }
}

impl HeaderStore for DbHeadersStore {
    fn insert(&self, hash: Hash, header: Arc<Header>, block_level: u8) -> Result<(), StoreError> {
        if self.headers_access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.compact_headers_access.write(
            DirectDbWriter::new(&self.db),
            hash,
            CompactHeaderData {
                timestamp: header.timestamp(),
                difficulty: header.difficulty(),
            },
        )?;
        self.headers_access.write(
            DirectDbWriter::new(&self.db),
            hash,
            HeaderWithBlockLevel {
                header,
                block_level,
            },
        )?;
        Ok(())
    }
}
