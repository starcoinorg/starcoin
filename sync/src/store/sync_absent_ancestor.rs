use std::sync::Arc;

use anyhow::format_err;
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_dag::{
    consensusdb::{
        prelude::{CachedDbAccess, DirectDbWriter, StoreError},
        schema::{KeyCodec, ValueCodec},
    },
    define_schema,
};
use starcoin_storage::{db_storage::{DBStorage, SchemaIterator}, storage};
use starcoin_types::block::Block;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DagSyncBlock {
    pub block: Option<Block>,
    pub children: Vec<HashValue>,
}

impl Default for DagSyncBlock {
    fn default() -> Self {
        Self {
            block: None,
            children: vec![],
        }
    }
}

/// Reader API for `AbsentDagBlockStoreReader`.
pub trait AbsentDagBlockStoreReader {
    fn get_absent_block(&self, count: usize) -> anyhow::Result<Vec<DagSyncBlock>>;
    fn get_absent_block_by_id(&self, hash: HashValue) -> anyhow::Result<DagSyncBlock, StoreError>;
    fn iter_at_first(&self) -> anyhow::Result<SchemaIterator<HashValue, DagSyncBlock>>;
}

pub trait AbsentDagBlockStoreWriter {
    fn save_absent_block(&mut self, blocks: Vec<DagSyncBlock>) -> anyhow::Result<()>;
    fn delete_absent_block(&mut self, hash: HashValue) -> anyhow::Result<()>;
}

pub(crate) const SYNC_ABSENT_BLOCK_CF: &str = "sync-absent-block";

define_schema!(
    SyncAbsentBlock,
    HashValue,
    DagSyncBlock,
    SYNC_ABSENT_BLOCK_CF
);

impl KeyCodec<SyncAbsentBlock> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        Ok(self.encode().map_err(|e| StoreError::EncodeError(e.to_string()))?)
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        // HashValue::from_slice(data).map_err(|e| StoreError::DecodeError(e.to_string()))
        let s = HashValue::decode(data).map_err(|e| StoreError::DecodeError(e.to_string()))?;
        Ok(s) 
    }
}

impl ValueCodec<SyncAbsentBlock> for DagSyncBlock {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        let s = bcs_ext::to_bytes(self).map_err(|e| StoreError::EncodeError(e.to_string()))?;
        let obj: DagSyncBlock = bcs_ext::from_bytes(&s.clone()).map_err(|e| StoreError::DecodeError(e.to_string()))?;
        assert_eq!(self.clone(), obj);
        Ok(s)
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        let s: DagSyncBlock = bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))?;
        Ok(s)
    }
}

impl storage::ValueCodec for DagSyncBlock {
    fn encode_value(&self) -> anyhow::Result<Vec<u8>> {
        bcs_ext::to_bytes(self)
    }

    fn decode_value(data: &[u8]) -> anyhow::Result<Self> {
        bcs_ext::from_bytes(data)
    }
}

#[derive(Clone)]
pub struct SyncAbsentBlockStore {
    db: Arc<DBStorage>,
    cache_access: CachedDbAccess<SyncAbsentBlock>,
}

impl SyncAbsentBlockStore {
    pub fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            db: db.clone(),
            cache_access: CachedDbAccess::new(db, cache_size),
        }
    }

    pub fn clone_with_new_cache(&self, cache_size: usize) -> Self {
        Self::new(self.db.clone(), cache_size)
    }
}

impl AbsentDagBlockStoreReader for SyncAbsentBlockStore {
    fn get_absent_block(&self, count: usize) -> anyhow::Result<Vec<DagSyncBlock>> {
        let mut blocks = Vec::new();

        let mut iter = self
            .db
            .iter::<HashValue, DagSyncBlock>(SYNC_ABSENT_BLOCK_CF)?;
        iter.seek_to_first();
        for _i in 0..count {
            if let Some(result) = iter.next() {
                let (_, block) = result?;
                blocks.push(block);
            } else {
                break;
            }
        }
        anyhow::Result::Ok(blocks)
    }

    fn get_absent_block_by_id(&self, hash: HashValue) -> anyhow::Result<DagSyncBlock, StoreError> {
        self.cache_access
            .read(hash)
    }
    
    fn iter_at_first(&self) -> anyhow::Result<SchemaIterator<HashValue, DagSyncBlock>> {
        let mut iter = self.db.iter::<HashValue, DagSyncBlock>(SYNC_ABSENT_BLOCK_CF)?;
        iter.seek_to_first();
        Ok(iter)
    }
}

impl AbsentDagBlockStoreWriter for SyncAbsentBlockStore {
    fn delete_absent_block(&mut self, hash: HashValue) -> anyhow::Result<()> {
        self.cache_access
            .delete(DirectDbWriter::new(&self.db), hash)
            .map_err(|e| format_err!("failed to delete absent block: {:?}", e))
    }

    fn save_absent_block(&mut self, blocks: Vec<DagSyncBlock>) -> anyhow::Result<()> {
        for block in blocks {
            self.cache_access.write(
                DirectDbWriter::new(&self.db),
                block
                    .block
                    .as_ref()
                    .ok_or_else(|| format_err!("block in sync dag block should not be none!"))?
                    .header()
                    .id(),
                block.clone(),
            )?;
        }
        Ok(())
    }
}
/////////////////////////////////////////////////////////////////////////////

// #[cfg(test)]
// mod tests {
//     use std::ops::DerefMut;

//     use super::*;
//     use crate::consensusdb::prelude::{FlexiDagStorage, FlexiDagStorageConfig};

//     #[test]
//     fn test_db_relations_store() {
//         let db_tempdir = tempfile::tempdir().unwrap();
//         let config = FlexiDagStorageConfig::new();

//         let db = FlexiDagStorage::create_from_path(db_tempdir.path(), config)
//             .expect("failed to create flexidag storage");
//         test_relations_store(db.relations_store.write().deref_mut().clone());
//     }

//     fn test_relations_store<T: RelationsStore>(store: T) {
//         let parents = [
//             (1, vec![]),
//             (2, vec![1]),
//             (3, vec![1]),
//             (4, vec![2, 3]),
//             (5, vec![1, 4]),
//         ];
//         for (i, vec) in parents.iter().cloned() {
//             store
//                 .insert(
//                     i.into(),
//                     BlockHashes::new(vec.iter().copied().map(Hash::from).collect()),
//                 )
//                 .unwrap();
//         }

//         let expected_children = [
//             (1, vec![2, 3, 5]),
//             (2, vec![4]),
//             (3, vec![4]),
//             (4, vec![5]),
//             (5, vec![]),
//         ];
//         for (i, vec) in expected_children {
//             assert!(store
//                 .get_children(i.into())
//                 .unwrap()
//                 .iter()
//                 .copied()
//                 .eq(vec.iter().copied().map(Hash::from)));
//         }

//         for (i, vec) in parents {
//             assert!(store
//                 .get_parents(i.into())
//                 .unwrap()
//                 .iter()
//                 .copied()
//                 .eq(vec.iter().copied().map(Hash::from)));
//         }
//     }
// }
