use super::schema::{KeyCodec, ValueCodec};
use super::{
    db::DBStorage,
    error::StoreError,
    prelude::{CachedDbAccess, DirectDbWriter},
    writer::BatchDbWriter,
};
use crate::define_schema;
use starcoin_types::blockhash::{
    BlockHashMap, BlockHashes, BlockLevel, BlueWorkType, HashKTypeMap,
};

use crate::dag::types::{
    ghostdata::{CompactGhostdagData, GhostdagData},
    ordering::SortableBlock,
};
use itertools::{
    EitherOrBoth::{Both, Left, Right},
    Itertools,
};
use rocksdb::WriteBatch;
use starcoin_crypto::HashValue as Hash;
use std::{cell::RefCell, cmp, iter::once, sync::Arc};

pub trait GhostdagStoreReader {
    fn get_blue_score(&self, hash: Hash) -> Result<u64, StoreError>;
    fn get_blue_work(&self, hash: Hash) -> Result<BlueWorkType, StoreError>;
    fn get_selected_parent(&self, hash: Hash) -> Result<Hash, StoreError>;
    fn get_mergeset_blues(&self, hash: Hash) -> Result<BlockHashes, StoreError>;
    fn get_mergeset_reds(&self, hash: Hash) -> Result<BlockHashes, StoreError>;
    fn get_blues_anticone_sizes(&self, hash: Hash) -> Result<HashKTypeMap, StoreError>;

    /// Returns full block data for the requested hash
    fn get_data(&self, hash: Hash) -> Result<Arc<GhostdagData>, StoreError>;

    fn get_compact_data(&self, hash: Hash) -> Result<CompactGhostdagData, StoreError>;

    /// Check if the store contains data for the requested hash
    fn has(&self, hash: Hash) -> Result<bool, StoreError>;
}

pub trait GhostdagStore: GhostdagStoreReader {
    /// Insert GHOSTDAG data for block `hash` into the store. Note that GHOSTDAG data
    /// is added once and never modified, so no need for specific setters for each element.
    /// Additionally, this means writes are semantically "append-only", which is why
    /// we can keep the `insert` method non-mutable on self. See "Parallel Processing.md" for an overview.
    fn insert(&self, hash: Hash, data: Arc<GhostdagData>) -> Result<(), StoreError>;
}

pub struct GhostDagDataWrapper(GhostdagData);

impl From<GhostdagData> for GhostDagDataWrapper {
    fn from(value: GhostdagData) -> Self {
        Self(value)
    }
}

impl GhostDagDataWrapper {
    /// Returns an iterator to the mergeset in ascending blue work order (tie-breaking by hash)
    pub fn ascending_mergeset_without_selected_parent<'a>(
        &'a self,
        store: &'a (impl GhostdagStoreReader + ?Sized),
    ) -> impl Iterator<Item = Result<SortableBlock, StoreError>> + '_ {
        self.0
            .mergeset_blues
            .iter()
            .skip(1) // Skip the selected parent
            .cloned()
            .map(|h| {
                store
                    .get_blue_work(h)
                    .map(|blue| SortableBlock::new(h, blue))
            })
            .merge_join_by(
                self.0
                    .mergeset_reds
                    .iter()
                    .cloned()
                    .map(|h| store.get_blue_work(h).map(|red| SortableBlock::new(h, red))),
                |a, b| match (a, b) {
                    (Ok(a), Ok(b)) => a.cmp(b),
                    (Err(_), Ok(_)) => cmp::Ordering::Less, // select left Err node
                    (Ok(_), Err(_)) => cmp::Ordering::Greater, // select right Err node
                    (Err(_), Err(_)) => cmp::Ordering::Equal, // remove both Err nodes
                },
            )
            .map(|r| match r {
                Left(b) | Right(b) => b,
                Both(c, _) => Err(StoreError::DAGDupBlocksError(format!("{c:?}"))),
            })
    }

    /// Returns an iterator to the mergeset in descending blue work order (tie-breaking by hash)
    pub fn descending_mergeset_without_selected_parent<'a>(
        &'a self,
        store: &'a (impl GhostdagStoreReader + ?Sized),
    ) -> impl Iterator<Item = Result<SortableBlock, StoreError>> + '_ {
        self.0
            .mergeset_blues
            .iter()
            .skip(1) // Skip the selected parent
            .rev() // Reverse since blues and reds are stored with ascending blue work order
            .cloned()
            .map(|h| {
                store
                    .get_blue_work(h)
                    .map(|blue| SortableBlock::new(h, blue))
            })
            .merge_join_by(
                self.0
                    .mergeset_reds
                    .iter()
                    .rev() // Reverse
                    .cloned()
                    .map(|h| store.get_blue_work(h).map(|red| SortableBlock::new(h, red))),
                |a, b| match (b, a) {
                    (Ok(b), Ok(a)) => b.cmp(a),
                    (Err(_), Ok(_)) => cmp::Ordering::Less, // select left Err node
                    (Ok(_), Err(_)) => cmp::Ordering::Greater, // select right Err node
                    (Err(_), Err(_)) => cmp::Ordering::Equal, // select both Err nodes
                }, // Reverse
            )
            .map(|r| match r {
                Left(b) | Right(b) => b,
                Both(c, _) => Err(StoreError::DAGDupBlocksError(format!("{c:?}"))),
            })
    }

    /// Returns an iterator to the mergeset in topological consensus order -- starting with the selected parent,
    /// and adding the mergeset in increasing blue work order. Note that this is a topological order even though
    /// the selected parent has highest blue work by def -- since the mergeset is in its anticone.
    pub fn consensus_ordered_mergeset<'a>(
        &'a self,
        store: &'a (impl GhostdagStoreReader + ?Sized),
    ) -> impl Iterator<Item = Result<Hash, StoreError>> + '_ {
        once(Ok(self.0.selected_parent)).chain(
            self.ascending_mergeset_without_selected_parent(store)
                .map(|s| s.map(|s| s.hash)),
        )
    }

    /// Returns an iterator to the mergeset in topological consensus order without the selected parent
    pub fn consensus_ordered_mergeset_without_selected_parent<'a>(
        &'a self,
        store: &'a (impl GhostdagStoreReader + ?Sized),
    ) -> impl Iterator<Item = Result<Hash, StoreError>> + '_ {
        self.ascending_mergeset_without_selected_parent(store)
            .map(|s| s.map(|s| s.hash))
    }
}

pub(crate) const GHOST_DAG_STORE_CF: &str = "block-ghostdag-data";
pub(crate) const COMPACT_GHOST_DAG_STORE_CF: &str = "compact-block-ghostdag-data";

define_schema!(GhostDag, Hash, Arc<GhostdagData>, GHOST_DAG_STORE_CF);
define_schema!(
    CompactGhostDag,
    Hash,
    CompactGhostdagData,
    COMPACT_GHOST_DAG_STORE_CF
);

impl KeyCodec<GhostDag> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Hash::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<GhostDag> for Arc<GhostdagData> {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(&self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

impl KeyCodec<CompactGhostDag> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Hash::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<CompactGhostDag> for CompactGhostdagData {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(&self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

/// A DB + cache implementation of `GhostdagStore` trait, with concurrency support.
#[derive(Clone)]
pub struct DbGhostdagStore {
    db: Arc<DBStorage>,
    level: BlockLevel,
    access: CachedDbAccess<GhostDag>,
    compact_access: CachedDbAccess<CompactGhostDag>,
}

impl DbGhostdagStore {
    pub fn new(db: Arc<DBStorage>, level: BlockLevel, cache_size: u64) -> Self {
        Self {
            db: Arc::clone(&db),
            level,
            access: CachedDbAccess::new(db.clone(), cache_size),
            compact_access: CachedDbAccess::new(db, cache_size),
        }
    }

    pub fn clone_with_new_cache(&self, cache_size: u64) -> Self {
        Self::new(Arc::clone(&self.db), self.level, cache_size)
    }

    pub fn insert_batch(
        &self,
        batch: &mut WriteBatch,
        hash: Hash,
        data: &Arc<GhostdagData>,
    ) -> Result<(), StoreError> {
        if self.access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.access
            .write(BatchDbWriter::new(batch), hash, data.clone())?;
        self.compact_access.write(
            BatchDbWriter::new(batch),
            hash,
            CompactGhostdagData {
                blue_score: data.blue_score,
                blue_work: data.blue_work,
                selected_parent: data.selected_parent,
            },
        )?;
        Ok(())
    }
}

impl GhostdagStoreReader for DbGhostdagStore {
    fn get_blue_score(&self, hash: Hash) -> Result<u64, StoreError> {
        Ok(self.access.read(hash)?.blue_score)
    }

    fn get_blue_work(&self, hash: Hash) -> Result<BlueWorkType, StoreError> {
        Ok(self.access.read(hash)?.blue_work)
    }

    fn get_selected_parent(&self, hash: Hash) -> Result<Hash, StoreError> {
        Ok(self.access.read(hash)?.selected_parent)
    }

    fn get_mergeset_blues(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        Ok(Arc::clone(&self.access.read(hash)?.mergeset_blues))
    }

    fn get_mergeset_reds(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        Ok(Arc::clone(&self.access.read(hash)?.mergeset_reds))
    }

    fn get_blues_anticone_sizes(&self, hash: Hash) -> Result<HashKTypeMap, StoreError> {
        Ok(Arc::clone(&self.access.read(hash)?.blues_anticone_sizes))
    }

    fn get_data(&self, hash: Hash) -> Result<Arc<GhostdagData>, StoreError> {
        self.access.read(hash)
    }

    fn get_compact_data(&self, hash: Hash) -> Result<CompactGhostdagData, StoreError> {
        self.compact_access.read(hash)
    }

    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        self.access.has(hash)
    }
}

impl GhostdagStore for DbGhostdagStore {
    fn insert(&self, hash: Hash, data: Arc<GhostdagData>) -> Result<(), StoreError> {
        if self.access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.access
            .write(DirectDbWriter::new(&self.db), hash, data.clone())?;
        if self.compact_access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.compact_access.write(
            DirectDbWriter::new(&self.db),
            hash,
            CompactGhostdagData {
                blue_score: data.blue_score,
                blue_work: data.blue_work,
                selected_parent: data.selected_parent,
            },
        )?;
        Ok(())
    }
}

/// An in-memory implementation of `GhostdagStore` trait to be used for tests.
/// Uses `RefCell` for interior mutability in order to workaround `insert`
/// being non-mutable.
pub struct MemoryGhostdagStore {
    blue_score_map: RefCell<BlockHashMap<u64>>,
    blue_work_map: RefCell<BlockHashMap<BlueWorkType>>,
    selected_parent_map: RefCell<BlockHashMap<Hash>>,
    mergeset_blues_map: RefCell<BlockHashMap<BlockHashes>>,
    mergeset_reds_map: RefCell<BlockHashMap<BlockHashes>>,
    blues_anticone_sizes_map: RefCell<BlockHashMap<HashKTypeMap>>,
}

impl MemoryGhostdagStore {
    pub fn new() -> Self {
        Self {
            blue_score_map: RefCell::new(BlockHashMap::new()),
            blue_work_map: RefCell::new(BlockHashMap::new()),
            selected_parent_map: RefCell::new(BlockHashMap::new()),
            mergeset_blues_map: RefCell::new(BlockHashMap::new()),
            mergeset_reds_map: RefCell::new(BlockHashMap::new()),
            blues_anticone_sizes_map: RefCell::new(BlockHashMap::new()),
        }
    }
}

impl Default for MemoryGhostdagStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GhostdagStore for MemoryGhostdagStore {
    fn insert(&self, hash: Hash, data: Arc<GhostdagData>) -> Result<(), StoreError> {
        if self.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.blue_score_map
            .borrow_mut()
            .insert(hash, data.blue_score);
        self.blue_work_map.borrow_mut().insert(hash, data.blue_work);
        self.selected_parent_map
            .borrow_mut()
            .insert(hash, data.selected_parent);
        self.mergeset_blues_map
            .borrow_mut()
            .insert(hash, data.mergeset_blues.clone());
        self.mergeset_reds_map
            .borrow_mut()
            .insert(hash, data.mergeset_reds.clone());
        self.blues_anticone_sizes_map
            .borrow_mut()
            .insert(hash, data.blues_anticone_sizes.clone());
        Ok(())
    }
}

impl GhostdagStoreReader for MemoryGhostdagStore {
    fn get_blue_score(&self, hash: Hash) -> Result<u64, StoreError> {
        match self.blue_score_map.borrow().get(&hash) {
            Some(blue_score) => Ok(*blue_score),
            None => Err(StoreError::KeyNotFound(hash.to_string())),
        }
    }

    fn get_blue_work(&self, hash: Hash) -> Result<BlueWorkType, StoreError> {
        match self.blue_work_map.borrow().get(&hash) {
            Some(blue_work) => Ok(*blue_work),
            None => Err(StoreError::KeyNotFound(hash.to_string())),
        }
    }

    fn get_selected_parent(&self, hash: Hash) -> Result<Hash, StoreError> {
        match self.selected_parent_map.borrow().get(&hash) {
            Some(selected_parent) => Ok(*selected_parent),
            None => Err(StoreError::KeyNotFound(hash.to_string())),
        }
    }

    fn get_mergeset_blues(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        match self.mergeset_blues_map.borrow().get(&hash) {
            Some(mergeset_blues) => Ok(BlockHashes::clone(mergeset_blues)),
            None => Err(StoreError::KeyNotFound(hash.to_string())),
        }
    }

    fn get_mergeset_reds(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        match self.mergeset_reds_map.borrow().get(&hash) {
            Some(mergeset_reds) => Ok(BlockHashes::clone(mergeset_reds)),
            None => Err(StoreError::KeyNotFound(hash.to_string())),
        }
    }

    fn get_blues_anticone_sizes(&self, hash: Hash) -> Result<HashKTypeMap, StoreError> {
        match self.blues_anticone_sizes_map.borrow().get(&hash) {
            Some(sizes) => Ok(HashKTypeMap::clone(sizes)),
            None => Err(StoreError::KeyNotFound(hash.to_string())),
        }
    }

    fn get_data(&self, hash: Hash) -> Result<Arc<GhostdagData>, StoreError> {
        if !self.has(hash)? {
            return Err(StoreError::KeyNotFound(hash.to_string()));
        }
        Ok(Arc::new(GhostdagData::new(
            self.blue_score_map.borrow()[&hash],
            self.blue_work_map.borrow()[&hash],
            self.selected_parent_map.borrow()[&hash],
            self.mergeset_blues_map.borrow()[&hash].clone(),
            self.mergeset_reds_map.borrow()[&hash].clone(),
            self.blues_anticone_sizes_map.borrow()[&hash].clone(),
        )))
    }

    fn get_compact_data(&self, hash: Hash) -> Result<CompactGhostdagData, StoreError> {
        Ok(self.get_data(hash)?.to_compact())
    }

    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        Ok(self.blue_score_map.borrow().contains_key(&hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_types::blockhash::BlockHashSet;
    use std::iter::once;

    #[test]
    fn test_mergeset_iterators() {
        let store = MemoryGhostdagStore::new();

        let factory = |w: u64| {
            Arc::new(GhostdagData {
                blue_score: Default::default(),
                blue_work: w.into(),
                selected_parent: Default::default(),
                mergeset_blues: Default::default(),
                mergeset_reds: Default::default(),
                blues_anticone_sizes: Default::default(),
            })
        };

        // Blues
        store.insert(1.into(), factory(2)).unwrap();
        store.insert(2.into(), factory(7)).unwrap();
        store.insert(3.into(), factory(11)).unwrap();

        // Reds
        store.insert(4.into(), factory(4)).unwrap();
        store.insert(5.into(), factory(9)).unwrap();
        store.insert(6.into(), factory(11)).unwrap(); // Tie-breaking case

        let mut data = GhostdagData::new_with_selected_parent(1.into(), 5);
        data.add_blue(2.into(), Default::default(), &Default::default());
        data.add_blue(3.into(), Default::default(), &Default::default());

        data.add_red(4.into());
        data.add_red(5.into());
        data.add_red(6.into());

        let wrapper: GhostDagDataWrapper = data.clone().into();

        let mut expected: Vec<Hash> = vec![4.into(), 2.into(), 5.into(), 3.into(), 6.into()];
        assert_eq!(
            expected,
            wrapper
                .ascending_mergeset_without_selected_parent(&store)
                .filter_map(|b| b.map(|b| b.hash).ok())
                .collect::<Vec<Hash>>()
        );

        itertools::assert_equal(
            once(1.into()).chain(expected.iter().cloned()),
            wrapper
                .consensus_ordered_mergeset(&store)
                .filter_map(|b| b.ok()),
        );

        expected.reverse();
        assert_eq!(
            expected,
            wrapper
                .descending_mergeset_without_selected_parent(&store)
                .filter_map(|b| b.map(|b| b.hash).ok())
                .collect::<Vec<Hash>>()
        );

        // Use sets since the below functions have no order guarantee
        let expected = BlockHashSet::from_iter([4.into(), 2.into(), 5.into(), 3.into(), 6.into()]);
        assert_eq!(
            expected,
            data.unordered_mergeset_without_selected_parent()
                .collect::<BlockHashSet>()
        );

        let expected =
            BlockHashSet::from_iter([1.into(), 4.into(), 2.into(), 5.into(), 3.into(), 6.into()]);
        assert_eq!(
            expected,
            data.unordered_mergeset().collect::<BlockHashSet>()
        );
    }
}
