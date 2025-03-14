use super::schema::{KeyCodec, ValueCodec};
use super::writer::BatchDbWriter;
use super::{
    db::DBStorage,
    prelude::{CachedDbAccess, StoreError},
};
use crate::define_schema;
use starcoin_crypto::HashValue as Hash;
use starcoin_storage::batch::{WriteBatch, WriteBatchData, WriteBatchWithColumn};
use starcoin_storage::storage::{InnerStore, WriteOp};
use starcoin_types::blockhash::{BlockHashes, BlockLevel};
use std::collections::HashMap;
use std::sync::Arc;
/// Reader API for `RelationsStore`.
pub trait RelationsStoreReader {
    fn get_parents(&self, hash: Hash) -> Result<BlockHashes, StoreError>;
    fn get_children(&self, hash: Hash) -> Result<BlockHashes, StoreError>;
    fn has(&self, hash: Hash) -> Result<bool, StoreError>;
}

/// Write API for `RelationsStore`. The insert function is deliberately `mut`
/// since it modifies the children arrays for previously added parents which is
/// non-append-only and thus needs to be guarded.
pub trait RelationsStore: RelationsStoreReader {
    /// Inserts `parents` into a new store entry for `hash`, and for each `parent ∈ parents` adds `hash` to `parent.children`
    fn insert(&self, hash: Hash, parents: BlockHashes) -> Result<(), StoreError>;
}

pub(crate) const PARENTS_CF: &str = "block-parents";
pub(crate) const CHILDREN_CF: &str = "block-children";

define_schema!(RelationParent, Hash, Arc<Vec<Hash>>, PARENTS_CF);
define_schema!(RelationChildren, Hash, Arc<Vec<Hash>>, CHILDREN_CF);

impl KeyCodec<RelationParent> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Self::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl ValueCodec<RelationParent> for Arc<Vec<Hash>> {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
impl KeyCodec<RelationChildren> for Hash {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        Self::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

impl ValueCodec<RelationChildren> for Arc<Vec<Hash>> {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}

/// A DB + cache implementation of `RelationsStore` trait, with concurrent readers support.
#[derive(Clone)]
pub struct DbRelationsStore {
    db: Arc<DBStorage>,
    level: BlockLevel,
    parents_access: CachedDbAccess<RelationParent>,
    children_access: CachedDbAccess<RelationChildren>,
}

impl DbRelationsStore {
    pub fn new(db: Arc<DBStorage>, level: BlockLevel, cache_size: usize) -> Self {
        Self {
            db: Arc::clone(&db),
            level,
            parents_access: CachedDbAccess::new(Arc::clone(&db), cache_size),
            children_access: CachedDbAccess::new(db, cache_size),
        }
    }

    pub fn clone_with_new_cache(&self, cache_size: usize) -> Self {
        Self::new(Arc::clone(&self.db), self.level, cache_size)
    }

    pub fn insert_batch(
        &mut self,
        batch: &mut rocksdb::WriteBatch,
        hash: Hash,
        parents: BlockHashes,
    ) -> Result<(), StoreError> {
        if self.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }

        // Insert a new entry for `hash`
        self.parents_access
            .write(BatchDbWriter::new(batch, &self.db), hash, parents.clone())?;

        // The new hash has no children yet
        self.children_access.write(
            BatchDbWriter::new(batch, &self.db),
            hash,
            BlockHashes::new(Vec::new()),
        )?;

        // Update `children` for each parent
        for parent in parents.iter().cloned() {
            let mut children = (*self.get_children(parent)?).clone();
            children.push(hash);
            self.children_access.write(
                BatchDbWriter::new(batch, &self.db),
                parent,
                BlockHashes::new(children),
            )?;
        }

        Ok(())
    }
}

impl RelationsStoreReader for DbRelationsStore {
    fn get_parents(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        self.parents_access.read(hash)
    }

    fn get_children(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        self.children_access.read(hash)
    }

    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        if self.parents_access.has(hash)? {
            debug_assert!(self.children_access.has(hash)?);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl RelationsStore for DbRelationsStore {
    fn insert(&self, hash: Hash, parents: BlockHashes) -> Result<(), StoreError> {
        if self.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }

        let mut parent_to_children = HashMap::new();
        parent_to_children.insert(hash, vec![]);

        for parent in parents.iter().cloned() {
            let mut children = match self.get_children(parent) {
                Ok(children) => (*children).clone(),
                Err(e) => match e {
                    StoreError::KeyNotFound(_) => vec![],
                    _ => return std::result::Result::Err(e),
                },
            };
            children.push(hash);
            parent_to_children.insert(parent, children);
        }

        let batch = WriteBatchWithColumn {
            data: vec![
                WriteBatchData {
                    column: PARENTS_CF.to_string(),
                    row_data: WriteBatch::new_with_rows(vec![(
                        hash.to_vec(),
                        WriteOp::Value(
                            <Arc<Vec<Hash>> as ValueCodec<RelationParent>>::encode_value(&parents)?,
                        ),
                    )]),
                },
                WriteBatchData {
                    column: CHILDREN_CF.to_string(),
                    row_data: WriteBatch::new_with_rows(
                        parent_to_children
                            .iter()
                            .map(|(key, value)| {
                                std::result::Result::Ok((
                                    key.to_vec(),
                                    WriteOp::Value(<Arc<Vec<Hash>> as ValueCodec<
                                        RelationChildren,
                                    >>::encode_value(
                                        &Arc::new(value.clone())
                                    )?),
                                ))
                            })
                            .collect::<std::result::Result<Vec<_>, StoreError>>()?,
                    ),
                },
            ],
        };
        self.db
            .write_batch_with_column(batch)
            .map_err(|e| StoreError::DBIoError(format!("Failed to write batch when writing batch with column for the dag releationship: {:?}", e)))?;

        self.parents_access.flush_cache(&[(hash, parents)])?;
        self.children_access.flush_cache(
            &parent_to_children
                .into_iter()
                .map(|(key, value)| (key, BlockHashes::new(value)))
                .collect::<Vec<_>>(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::DerefMut;

    use super::*;
    use crate::consensusdb::prelude::{FlexiDagStorage, FlexiDagStorageConfig};

    #[test]
    fn test_db_relations_store() {
        let db_tempdir = tempfile::tempdir().unwrap();
        let config = FlexiDagStorageConfig::new();

        let db = FlexiDagStorage::create_from_path(db_tempdir.path(), config)
            .expect("failed to create flexidag storage");
        test_relations_store(db.relations_store.write().deref_mut().clone());
    }

    fn test_relations_store<T: RelationsStore>(store: T) {
        let parents = [
            (1, vec![]),
            (2, vec![1]),
            (3, vec![1]),
            (4, vec![2, 3]),
            (5, vec![1, 4]),
        ];
        for (i, vec) in parents.iter().cloned() {
            store
                .insert(
                    i.into(),
                    BlockHashes::new(vec.iter().copied().map(Hash::from).collect()),
                )
                .unwrap();
        }

        let expected_children = [
            (1, vec![2, 3, 5]),
            (2, vec![4]),
            (3, vec![4]),
            (4, vec![5]),
            (5, vec![]),
        ];
        for (i, vec) in expected_children {
            assert!(store
                .get_children(i.into())
                .unwrap()
                .iter()
                .copied()
                .eq(vec.iter().copied().map(Hash::from)));
        }

        for (i, vec) in parents {
            assert!(store
                .get_parents(i.into())
                .unwrap()
                .iter()
                .copied()
                .eq(vec.iter().copied().map(Hash::from)));
        }
    }
}
