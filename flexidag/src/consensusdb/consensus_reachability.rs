use super::{
    db::DBStorage,
    prelude::{BatchDbWriter, CachedDbAccess, CachedDbItem, DirectDbWriter, StoreError},
};
use starcoin_crypto::HashValue as Hash;

use crate::{
    consensusdb::schema::{KeyCodec, ValueCodec},
    define_schema,
    types::{interval::Interval, reachability::ReachabilityData},
};
use starcoin_types::blockhash::{self, BlockHashMap, BlockHashes};

use crate::consensusdb::prelude::DagCache;
use crate::consensusdb::schema::Schema;
use crate::consensusdb::set_access::DbSetAccess;
use parking_lot::{RwLockUpgradableReadGuard, RwLockWriteGuard};
use rocksdb::WriteBatch;
use std::{collections::hash_map::Entry::Vacant, sync::Arc};

/// Reader API for `ReachabilityStore`.
pub trait ReachabilityStoreReader {
    fn has(&self, hash: Hash) -> Result<bool, StoreError>;
    fn get_interval(&self, hash: Hash) -> Result<Interval, StoreError>;
    fn get_parent(&self, hash: Hash) -> Result<Hash, StoreError>;
    fn get_children(&self, hash: Hash) -> Result<BlockHashes, StoreError>;
    fn get_future_covering_set(&self, hash: Hash) -> Result<BlockHashes, StoreError>;
}

/// Write API for `ReachabilityStore`. All write functions are deliberately `mut`
/// since reachability writes are not append-only and thus need to be guarded.
pub trait ReachabilityStore: ReachabilityStoreReader {
    fn init(&mut self, origin: Hash, capacity: Interval) -> Result<(), StoreError>;
    fn insert(
        &mut self,
        hash: Hash,
        parent: Hash,
        interval: Interval,
        height: u64,
    ) -> Result<(), StoreError>;
    fn set_interval(&mut self, hash: Hash, interval: Interval) -> Result<(), StoreError>;
    fn append_child(&mut self, hash: Hash, child: Hash) -> Result<u64, StoreError>;
    fn insert_future_covering_item(
        &mut self,
        hash: Hash,
        fci: Hash,
        insertion_index: usize,
    ) -> Result<(), StoreError>;
    fn get_height(&self, hash: Hash) -> Result<u64, StoreError>;
    fn set_reindex_root(&mut self, root: Hash) -> Result<(), StoreError>;
    fn get_reindex_root(&self) -> Result<Hash, StoreError>;
}

pub const REINDEX_ROOT_KEY: &str = "reachability-reindex-root";
pub(crate) const REACHABILITY_DATA_CF: &str = "reachability-data";
pub(crate) const REACHABILITY_SET_DATA_CF: &str = "reachability-set-data";
pub(crate) const REACHABILITY_FCS_DATA_CF: &str = "reachability-fcs-data";
// TODO: explore perf to see if using fixed-length constants for store prefixes is preferable

define_schema!(
    Reachability,
    Hash,
    Arc<ReachabilityData>,
    REACHABILITY_DATA_CF
);
define_schema!(ReachabilityCache, Vec<u8>, Hash, REACHABILITY_DATA_CF);
define_schema!(ReachabilityChildren, Hash, Hash, REACHABILITY_SET_DATA_CF);
define_schema!(ReachabilityFcs, Hash, Hash, REACHABILITY_FCS_DATA_CF);

impl KeyCodec<Reachability> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Self::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<Reachability> for Arc<ReachabilityData> {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(&self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl KeyCodec<ReachabilityCache> for Vec<u8> {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Ok(data.to_vec())
    }
}
impl ValueCodec<ReachabilityCache> for Hash {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        Self::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

impl KeyCodec<ReachabilityChildren> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Hash::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<ReachabilityChildren> for Hash {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        Hash::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

impl KeyCodec<ReachabilityFcs> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Hash::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<ReachabilityFcs> for Hash {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        Hash::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

#[derive(Clone)]
struct DbReachabilitySet<S: Schema> {
    access: DbSetAccess<S>,
    cache: DagCache<S::Key, Arc<Vec<S::Value>>>,
}

impl<S: Schema> DbReachabilitySet<S> {
    fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            access: DbSetAccess::new(db),
            cache: DagCache::new_with_capacity(cache_size),
        }
    }

    fn read<K, F>(&self, key: S::Key, f: F) -> Result<Arc<Vec<S::Value>>, StoreError>
    where
        F: FnMut(&S::Value) -> K,
        K: Ord,
    {
        self.cache.get(&key).map_or_else(
            || {
                self.access.read(key.clone()).map(|mut v| {
                    v.sort_by_cached_key(f);
                    let v = Arc::new(v);
                    self.cache.insert(key, v.clone());
                    v
                })
            },
            Ok,
        )
    }

    fn initialize(&self, key: S::Key) {
        self.cache.insert(key, Arc::new(Vec::new()));
    }
}

/// A DB + cache implementation of `ReachabilityStore` trait, with concurrent readers support.
#[derive(Clone)]
pub struct DbReachabilityStore {
    db: Arc<DBStorage>,
    access: CachedDbAccess<Reachability>,
    children_store: DbReachabilitySet<ReachabilityChildren>,
    // future_covering_set
    fcs_store: DbReachabilitySet<ReachabilityFcs>,
    reindex_root: CachedDbItem<ReachabilityCache>,
}

impl DbReachabilityStore {
    pub fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            db: Arc::clone(&db),
            access: CachedDbAccess::new(Arc::clone(&db), cache_size),
            children_store: DbReachabilitySet::new(db.clone(), cache_size),
            fcs_store: DbReachabilitySet::new(db.clone(), cache_size),
            reindex_root: CachedDbItem::new(db, REINDEX_ROOT_KEY.as_bytes().to_vec()),
        }
    }

    pub fn clone_with_new_cache(&self, cache_size: usize) -> Self {
        Self::new(Arc::clone(&self.db), cache_size)
    }
}

impl ReachabilityStore for DbReachabilityStore {
    fn init(&mut self, origin: Hash, capacity: Interval) -> Result<(), StoreError> {
        debug_assert!(!self.access.has(origin)?);

        let data = Arc::new(ReachabilityData::new(
            Hash::new(blockhash::NONE),
            capacity,
            0,
        ));
        let mut batch = WriteBatch::default();
        self.access
            .write(BatchDbWriter::new(&mut batch, &self.db), origin, data)?;
        self.reindex_root
            .write(BatchDbWriter::new(&mut batch, &self.db), &origin)?;
        self.db
            .raw_write_batch(batch)
            .map_err(|e| StoreError::DBIoError(e.to_string()))?;

        self.children_store.initialize(origin);
        self.fcs_store.initialize(origin);

        Ok(())
    }

    fn insert(
        &mut self,
        hash: Hash,
        parent: Hash,
        interval: Interval,
        height: u64,
    ) -> Result<(), StoreError> {
        if self.access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        let data = Arc::new(ReachabilityData::new(parent, interval, height));
        self.access
            .write(DirectDbWriter::new(&self.db), hash, data)?;

        self.children_store.initialize(hash);
        self.fcs_store.initialize(hash);

        Ok(())
    }

    fn set_interval(&mut self, hash: Hash, interval: Interval) -> Result<(), StoreError> {
        let mut data = self.access.read(hash)?;
        Arc::make_mut(&mut data).interval = interval;
        self.access
            .write(DirectDbWriter::new(&self.db), hash, data)?;
        Ok(())
    }

    fn append_child(&mut self, hash: Hash, child: Hash) -> Result<u64, StoreError> {
        let mut data = self
            .children_store
            .read(hash, |&h| self.access.read(h).unwrap().interval)?;
        let data_mut = Arc::make_mut(&mut data);
        data_mut.push(child);
        self.children_store
            .access
            .write(DirectDbWriter::new(&self.db), hash, child)?;
        self.get_height(hash)
    }

    fn insert_future_covering_item(
        &mut self,
        hash: Hash,
        fci: Hash,
        insertion_index: usize,
    ) -> Result<(), StoreError> {
        let mut data = self
            .fcs_store
            .read(hash, |&h| self.access.read(h).unwrap().interval)?;
        let data_mut = Arc::make_mut(&mut data);
        data_mut.insert(insertion_index, fci);
        self.fcs_store
            .access
            .write(DirectDbWriter::new(&self.db), hash, fci)?;
        Ok(())
    }

    fn get_height(&self, hash: Hash) -> Result<u64, StoreError> {
        Ok(self.access.read(hash)?.height)
    }

    fn set_reindex_root(&mut self, root: Hash) -> Result<(), StoreError> {
        self.reindex_root
            .write(DirectDbWriter::new(&self.db), &root)
    }

    fn get_reindex_root(&self) -> Result<Hash, StoreError> {
        self.reindex_root.read()
    }
}

impl ReachabilityStoreReader for DbReachabilityStore {
    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        self.access.has(hash)
    }

    fn get_interval(&self, hash: Hash) -> Result<Interval, StoreError> {
        Ok(self.access.read(hash)?.interval)
    }

    fn get_parent(&self, hash: Hash) -> Result<Hash, StoreError> {
        Ok(self.access.read(hash)?.parent)
    }

    fn get_children(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        Ok(self
            .children_store
            .read(hash, |&h| self.access.read(h).unwrap().interval)?
            .clone())
    }

    fn get_future_covering_set(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        Ok(self
            .fcs_store
            .read(hash, |&h| self.access.read(h).unwrap().interval)?
            .clone())
    }
}

pub struct StagingReachabilityStore<'a> {
    store_read: RwLockUpgradableReadGuard<'a, DbReachabilityStore>,
    staging_writes: BlockHashMap<ReachabilityData>,
    staging_children: BlockHashMap<BlockHashes>,
    staging_fcs: BlockHashMap<BlockHashes>,
    staging_reindex_root: Option<Hash>,
}

impl<'a> StagingReachabilityStore<'a> {
    pub fn new(store_read: RwLockUpgradableReadGuard<'a, DbReachabilityStore>) -> Self {
        Self {
            store_read,
            staging_writes: BlockHashMap::new(),
            staging_children: BlockHashMap::new(),
            staging_fcs: BlockHashMap::new(),
            staging_reindex_root: None,
        }
    }

    pub fn commit(
        self,
        batch: &mut WriteBatch,
    ) -> Result<RwLockWriteGuard<'a, DbReachabilityStore>, StoreError> {
        let db = Arc::clone(&self.store_read.db);
        let mut store_write = RwLockUpgradableReadGuard::upgrade(self.store_read);
        for (k, v) in self.staging_writes {
            let data = Arc::new(v);
            store_write
                .access
                .write(BatchDbWriter::new(batch, &db), k, data)?
        }
        if let Some(root) = self.staging_reindex_root {
            store_write
                .reindex_root
                .write(BatchDbWriter::new(batch, &db), &root)?;
        }
        Ok(store_write)
    }
}

impl ReachabilityStore for StagingReachabilityStore<'_> {
    fn init(&mut self, origin: Hash, capacity: Interval) -> Result<(), StoreError> {
        self.insert(origin, Hash::new(blockhash::NONE), capacity, 0)?;
        self.set_reindex_root(origin)?;
        Ok(())
    }

    fn insert(
        &mut self,
        hash: Hash,
        parent: Hash,
        interval: Interval,
        height: u64,
    ) -> Result<(), StoreError> {
        if self.store_read.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        if let Vacant(e) = self.staging_writes.entry(hash) {
            e.insert(ReachabilityData::new(parent, interval, height));
            Ok(())
        } else {
            Err(StoreError::KeyAlreadyExists(hash.to_string()))
        }
    }

    fn set_interval(&mut self, hash: Hash, interval: Interval) -> Result<(), StoreError> {
        if let Some(data) = self.staging_writes.get_mut(&hash) {
            data.interval = interval;
            return Ok(());
        }

        let mut data = (*self.store_read.access.read(hash)?).clone();
        data.interval = interval;
        self.staging_writes.insert(hash, data);

        Ok(())
    }

    fn append_child(&mut self, hash: Hash, child: Hash) -> Result<u64, StoreError> {
        let height = self.get_height(hash)?;
        if let Some(data) = self.staging_children.get_mut(&hash) {
            if !data.contains(&child) {
                let data_write = Arc::make_mut(data);
                data_write.push(child);
            }
            return Ok(height);
        }

        let mut data = self
            .store_read
            .children_store
            .read(hash, |&h| self.get_interval(h).unwrap())?;
        if !data.contains(&child) {
            Arc::make_mut(&mut data).push(child);
            self.staging_children.insert(hash, data);
        }

        Ok(height)
    }

    fn insert_future_covering_item(
        &mut self,
        hash: Hash,
        fci: Hash,
        insertion_index: usize,
    ) -> Result<(), StoreError> {
        if let Some(data) = self.staging_fcs.get_mut(&hash) {
            let data = Arc::make_mut(data);
            data.insert(insertion_index, fci);
            return Ok(());
        }

        let mut data = self
            .store_read
            .fcs_store
            .read(hash, |&h| self.get_interval(h).unwrap())?;
        let data_mut = Arc::make_mut(&mut data);
        data_mut.insert(insertion_index, fci);
        self.staging_fcs.insert(hash, Arc::clone(&data));

        Ok(())
    }

    fn get_height(&self, hash: Hash) -> Result<u64, StoreError> {
        if let Some(data) = self.staging_writes.get(&hash) {
            Ok(data.height)
        } else {
            Ok(self.store_read.access.read(hash)?.height)
        }
    }

    fn set_reindex_root(&mut self, root: Hash) -> Result<(), StoreError> {
        self.staging_reindex_root = Some(root);
        Ok(())
    }

    fn get_reindex_root(&self) -> Result<Hash, StoreError> {
        if let Some(root) = self.staging_reindex_root {
            Ok(root)
        } else {
            Ok(self.store_read.get_reindex_root()?)
        }
    }
}

impl ReachabilityStoreReader for StagingReachabilityStore<'_> {
    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        Ok(self.staging_writes.contains_key(&hash) || self.store_read.access.has(hash)?)
    }

    fn get_interval(&self, hash: Hash) -> Result<Interval, StoreError> {
        if let Some(data) = self.staging_writes.get(&hash) {
            Ok(data.interval)
        } else {
            Ok(self.store_read.access.read(hash)?.interval)
        }
    }

    fn get_parent(&self, hash: Hash) -> Result<Hash, StoreError> {
        if let Some(data) = self.staging_writes.get(&hash) {
            Ok(data.parent)
        } else {
            Ok(self.store_read.access.read(hash)?.parent)
        }
    }

    fn get_children(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        if let Some(data) = self.staging_children.get(&hash) {
            Ok(BlockHashes::clone(data))
        } else {
            // todo: update staging_children?
            self.store_read
                .children_store
                .read(hash, |&h| self.get_interval(h).unwrap())
        }
    }

    fn get_future_covering_set(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        if let Some(data) = self.staging_fcs.get(&hash) {
            Ok(BlockHashes::clone(data))
        } else {
            // todo: need to update staging_fcs?
            self.store_read
                .fcs_store
                .read(hash, |&h| self.get_interval(h).unwrap())
        }
    }
}

pub struct MemoryReachabilityStore {
    map: BlockHashMap<ReachabilityData>,
    children_map: BlockHashMap<BlockHashes>,
    fcs_map: BlockHashMap<BlockHashes>,
    reindex_root: Option<Hash>,
}

impl Default for MemoryReachabilityStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryReachabilityStore {
    pub fn new() -> Self {
        Self {
            map: BlockHashMap::new(),
            children_map: Default::default(),
            fcs_map: Default::default(),
            reindex_root: None,
        }
    }

    fn get_data_mut(&mut self, hash: Hash) -> Result<&mut ReachabilityData, StoreError> {
        match self.map.get_mut(&hash) {
            Some(data) => Ok(data),
            None => Err(StoreError::KeyNotFound(hash.to_string())),
        }
    }

    fn get_data(&self, hash: Hash) -> Result<&ReachabilityData, StoreError> {
        match self.map.get(&hash) {
            Some(data) => Ok(data),
            None => Err(StoreError::KeyNotFound(hash.to_string())),
        }
    }
}

impl ReachabilityStore for MemoryReachabilityStore {
    fn init(&mut self, origin: Hash, capacity: Interval) -> Result<(), StoreError> {
        self.insert(origin, Hash::new(blockhash::NONE), capacity, 0)?;
        self.set_reindex_root(origin)?;
        Ok(())
    }

    fn insert(
        &mut self,
        hash: Hash,
        parent: Hash,
        interval: Interval,
        height: u64,
    ) -> Result<(), StoreError> {
        if let Vacant(e) = self.map.entry(hash) {
            e.insert(ReachabilityData::new(parent, interval, height));
            Ok(())
        } else {
            Err(StoreError::KeyAlreadyExists(hash.to_string()))
        }
    }

    fn set_interval(&mut self, hash: Hash, interval: Interval) -> Result<(), StoreError> {
        let data = self.get_data_mut(hash)?;
        data.interval = interval;
        Ok(())
    }

    fn append_child(&mut self, hash: Hash, child: Hash) -> Result<u64, StoreError> {
        let data = self.children_map.entry(hash).or_default();
        let data = Arc::make_mut(data);
        data.push(child);
        self.get_height(hash)
    }

    fn insert_future_covering_item(
        &mut self,
        hash: Hash,
        fci: Hash,
        insertion_index: usize,
    ) -> Result<(), StoreError> {
        let data = self.fcs_map.entry(hash).or_default();
        let data = Arc::make_mut(data);
        data.insert(insertion_index, fci);
        Ok(())
    }

    fn get_height(&self, hash: Hash) -> Result<u64, StoreError> {
        Ok(self.get_data(hash)?.height)
    }

    fn set_reindex_root(&mut self, root: Hash) -> Result<(), StoreError> {
        self.reindex_root = Some(root);
        Ok(())
    }

    fn get_reindex_root(&self) -> Result<Hash, StoreError> {
        match self.reindex_root {
            Some(root) => Ok(root),
            None => Err(StoreError::KeyNotFound(REINDEX_ROOT_KEY.to_string())),
        }
    }
}

impl ReachabilityStoreReader for MemoryReachabilityStore {
    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        Ok(self.map.contains_key(&hash))
    }

    fn get_interval(&self, hash: Hash) -> Result<Interval, StoreError> {
        Ok(self.get_data(hash)?.interval)
    }

    fn get_parent(&self, hash: Hash) -> Result<Hash, StoreError> {
        Ok(self.get_data(hash)?.parent)
    }

    fn get_children(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        Ok(Arc::clone(
            self.children_map
                .get(&hash)
                .unwrap_or(&BlockHashes::new(vec![])),
        ))
    }

    fn get_future_covering_set(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        Ok(Arc::clone(
            self.fcs_map.get(&hash).unwrap_or(&BlockHashes::new(vec![])),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_basics() {
        let mut store: Box<dyn ReachabilityStore> = Box::new(MemoryReachabilityStore::new());
        let (hash, parent) = (7.into(), 15.into());
        let interval = Interval::maximal();
        store.insert(hash, parent, interval, 5).unwrap();
        let height = store.append_child(hash, 31.into()).unwrap();
        assert_eq!(height, 5);
        let children = store.get_children(hash).unwrap();
        println!("{children:?}");
        store.get_interval(7.into()).unwrap();
        println!("{children:?}");
    }
}
