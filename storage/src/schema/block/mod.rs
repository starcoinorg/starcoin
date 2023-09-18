use anyhow::Result;
use rocksdb::ReadOptions;
use starcoin_crypto::HashValue;
use starcoin_schemadb::{
    db::DBStorage,
    iterator::SchemaIterator,
    schema::{KeyCodec, Schema},
    SchemaBatch,
};
use std::{marker::PhantomData, sync::Arc};
pub(crate) use {
    block_body::BlockBody as BlockBodySchema, block_header::BlockHeader as BlockHeaderSchema,
    block_inner::BlockInner as BlockInnerSchema,
    block_transaction::BlockTransaction as BlockTransactionSchema,
    block_transaction_info::BlockTransactionInfo as BlockTransactionInfoSchema,
    failed_block::FailedBlock as FailedBlockSchema,
};

mod block_body;
mod block_header;
mod block_inner;
mod block_transaction;
mod block_transaction_info;
mod failed_block;

pub(crate) type BlockInnerStorage = BlockStore<BlockInnerSchema>;
pub(crate) type BlockHeaderStorage = BlockStore<BlockHeaderSchema>;
pub(crate) type BlockBodyStorage = BlockStore<BlockBodySchema>;
pub(crate) type BlockTransactionsStorage = BlockStore<BlockTransactionSchema>;
pub(crate) type BlockTransactionInfosStorage = BlockStore<BlockTransactionInfoSchema>;
pub(crate) type FailedBlockStorage = BlockStore<FailedBlockSchema>;

// FixMe: Remove these functions
impl FailedBlockStorage {
    pub(crate) fn put_raw(&self, key: &HashValue, value: &[u8]) -> Result<()> {
        let raw = <HashValue as KeyCodec<FailedBlockSchema>>::encode_key(key)?;
        self.db
            .put_no_schema(FailedBlockSchema::COLUMN_FAMILY, &raw, value)
    }

    pub(crate) fn get_raw(&self, key: &HashValue) -> Result<Option<Vec<u8>>> {
        let raw = <HashValue as KeyCodec<FailedBlockSchema>>::encode_key(key)?;
        self.db
            .get_no_schema(FailedBlockSchema::COLUMN_FAMILY, &raw)
    }
}

#[derive(Clone)]
pub(crate) struct BlockStore<S: Schema> {
    db: Arc<DBStorage>,
    _phantom: PhantomData<S>,
}

impl<S: Schema> BlockStore<S> {
    pub(crate) fn new(db: &Arc<DBStorage>) -> Self {
        Self {
            db: Arc::clone(db),
            _phantom: Default::default(),
        }
    }

    pub(crate) fn get(&self, key: &S::Key) -> Result<Option<S::Value>> {
        self.db.get::<S>(key)
    }

    pub(crate) fn multi_get(&self, keys: &[S::Key]) -> Result<Vec<Option<S::Value>>> {
        self.db.batched_multi_get::<S>(keys)
    }

    pub(crate) fn put(&self, key: &S::Key, value: &S::Value) -> Result<()> {
        self.db.put::<S>(key, value)
    }

    pub(crate) fn remove(&self, key: &S::Key) -> Result<()> {
        self.db.remove::<S>(key)
    }

    pub(crate) fn iter(&self) -> Result<SchemaIterator<S>> {
        self.db.iter::<S>(ReadOptions::default())
    }

    pub(crate) fn put_batch(
        &self,
        batch: &SchemaBatch,
        key: &S::Key,
        val: &S::Value,
    ) -> Result<()> {
        batch.put::<S>(key, val)
    }

    pub(crate) fn remove_batch(&self, batch: &SchemaBatch, key: &S::Key) -> Result<()> {
        batch.delete::<S>(key)
    }
}
