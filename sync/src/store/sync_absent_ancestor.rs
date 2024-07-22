use std::sync::Arc;

use anyhow::format_err;
use bcs_ext::BCSCodec;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_dag::{
    consensusdb::{
        prelude::{CachedDbAccess, DirectDbWriter, StoreError},
        schema::{KeyCodec, ValueCodec},
    },
    define_schema,
};
use starcoin_storage::{
    db_storage::{DBStorage, SchemaIterator},
    storage,
};
use starcoin_types::block::{Block, BlockNumber};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct DagSyncBlock {
    pub block: Option<Block>,
}

#[derive(
    Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord, Default, Debug, Serialize, Deserialize,
)]
pub struct DagSyncBlockKey {
    pub number: BlockNumber,
    pub block_id: HashValue,
}

/// Reader API for `AbsentDagBlockStoreReader`.
pub trait AbsentDagBlockStoreReader {
    fn get_absent_block(&self, count: usize) -> anyhow::Result<Vec<DagSyncBlock>>;
    fn get_absent_block_by_id(
        &self,
        number: BlockNumber,
        block_id: HashValue,
    ) -> anyhow::Result<DagSyncBlock, StoreError>;
    fn iter_at_first(&self) -> anyhow::Result<SchemaIterator<Vec<u8>, Vec<u8>>>;
}

pub trait AbsentDagBlockStoreWriter {
    fn save_absent_block(&self, blocks: Vec<DagSyncBlock>) -> anyhow::Result<()>;
    fn delete_absent_block(&self, number: BlockNumber, block_id: HashValue) -> anyhow::Result<()>;
}

pub(crate) const SYNC_ABSENT_BLOCK_CF: &str = "sync-absent-block";

define_schema!(
    SyncAbsentBlock,
    DagSyncBlockKey,
    DagSyncBlock,
    SYNC_ABSENT_BLOCK_CF
);

impl KeyCodec<SyncAbsentBlock> for DagSyncBlockKey {
    fn encode_key(&self) -> Result<Vec<u8>, StoreError> {
        let mut buf = Vec::new();
        buf.write_u64::<BigEndian>(self.number).map_err(|e| {
            StoreError::EncodeError(format!(
                "failed to encode block number:{:?} for {:?}",
                self.number, e
            ))
        })?;
        buf.extend(self.block_id.encode().map_err(|e| {
            StoreError::EncodeError(format!(
                "failed to encode block id:{:?} for {:?}",
                self.block_id, e
            ))
        })?);
        Ok(buf)
    }

    fn decode_key(data: &[u8]) -> Result<Self, StoreError> {
        let (number_bytes, id_bytes) = data.split_at(8);
        let number = &mut &number_bytes[..];
        let number = number.read_u64::<BigEndian>().map_err(|e| {
            StoreError::DecodeError(format!("failed to decode block number for {:?}", e))
        })?;
        let block_id = HashValue::decode(id_bytes).map_err(|e| {
            StoreError::DecodeError(format!("failed to decode block id for {:?}", e))
        })?;
        Ok(Self { number, block_id })
    }
}

impl ValueCodec<SyncAbsentBlock> for DagSyncBlock {
    fn encode_value(&self) -> Result<Vec<u8>, StoreError> {
        bcs_ext::to_bytes(self).map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> Result<Self, StoreError> {
        bcs_ext::from_bytes(data).map_err(|e| StoreError::DecodeError(e.to_string()))
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

    fn get_absent_block_by_id(
        &self,
        number: BlockNumber,
        block_id: HashValue,
    ) -> anyhow::Result<DagSyncBlock, StoreError> {
        self.cache_access.read(DagSyncBlockKey { number, block_id })
    }

    fn iter_at_first(&self) -> anyhow::Result<SchemaIterator<Vec<u8>, Vec<u8>>> {
        let mut iter = self.db.iter::<Vec<u8>, Vec<u8>>(SYNC_ABSENT_BLOCK_CF)?;
        iter.seek_to_first();
        Ok(iter)
    }
}

impl AbsentDagBlockStoreWriter for SyncAbsentBlockStore {
    fn delete_absent_block(&self, number: BlockNumber, block_id: HashValue) -> anyhow::Result<()> {
        self.cache_access
            .delete(
                DirectDbWriter::new(&self.db),
                DagSyncBlockKey { number, block_id },
            )
            .map_err(|e| format_err!("failed to delete absent block: {:?}", e))
    }

    fn save_absent_block(&self, blocks: Vec<DagSyncBlock>) -> anyhow::Result<()> {
        for block in blocks {
            let ref_block = block
                .block
                .as_ref()
                .ok_or_else(|| format_err!("block in sync dag block should not be none!"))?;
            self.cache_access.write(
                DirectDbWriter::new(&self.db),
                DagSyncBlockKey {
                    number: ref_block.header().number(),
                    block_id: ref_block.header().id(),
                },
                block.clone(),
            )?;
        }
        Ok(())
    }
}
