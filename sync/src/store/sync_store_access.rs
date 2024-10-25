use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::schema::{KeyCodec, ValueCodec};
use starcoin_types::block::{Block, BlockNumber};
use std::collections::BTreeMap;

use super::{
    sync_absent_ancestor::{DagSyncBlock, DagSyncBlockKey},
    sync_dag_store::SyncDagStore,
};

pub type SyncStore<'a> =
    &'a mut dyn SyncStoreAccess<'a, Type = Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>>;

pub trait SyncStoreIterator<'a> {
    type Type: Iterator<Item = (Vec<u8>, Vec<u8>)>;

    fn iter(&'a self) -> Self::Type;
}

pub trait SyncStoreAccessBasic {
    fn insert(&mut self, block: Block) -> anyhow::Result<()>;
    fn get(&self, number: BlockNumber, block_id: HashValue) -> anyhow::Result<Option<Block>>;
    fn delete(&mut self, number: BlockNumber, block_id: HashValue) -> anyhow::Result<()>;
    fn delete_all(&mut self) -> anyhow::Result<()>;
}

pub trait SyncStoreAccess<'a>: SyncStoreAccessBasic + SyncStoreIterator<'a> {}

pub struct SyncStoreAccessMemory {
    store: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl SyncStoreAccessMemory {
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }
}

impl Default for SyncStoreAccessMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncStoreAccessBasic for SyncStoreAccessMemory {
    fn insert(&mut self, block: Block) -> anyhow::Result<()> {
        self.store.insert(
            DagSyncBlockKey {
                number: block.header().number(),
                block_id: block.id(),
            }
            .encode_key()?,
            DagSyncBlock { block: Some(block) }.encode_value()?,
        );
        anyhow::Ok(())
    }

    fn get(&self, number: BlockNumber, block_id: HashValue) -> anyhow::Result<Option<Block>> {
        match self
            .store
            .get(&DagSyncBlockKey { number, block_id }.encode_key()?)
        {
            Some(v) => anyhow::Result::Ok(DagSyncBlock::decode_value(v)?.block),
            None => anyhow::Result::Ok(None),
        }
    }

    fn delete(&mut self, number: BlockNumber, block_id: HashValue) -> anyhow::Result<()> {
        self.store
            .remove(&DagSyncBlockKey { number, block_id }.encode_key()?);
        anyhow::Ok(())
    }

    fn delete_all(&mut self) -> anyhow::Result<()> {
        self.store.clear();
        anyhow::Ok(())
    }
}

impl<'a> SyncStoreIterator<'a> for SyncStoreAccessMemory {
    type Type = Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>;

    fn iter(&'a self) -> Self::Type {
        Box::new(self.store.iter().map(|(k, v)| (k.clone(), v.clone())))
    }
}

impl SyncStoreAccess<'_> for SyncStoreAccessMemory {}

pub struct SyncStoreAccessDB {
    store: SyncDagStore,
}

impl SyncStoreAccessDB {
    pub fn new(store: SyncDagStore) -> Self {
        Self { store }
    }
}

impl SyncStoreAccessBasic for SyncStoreAccessDB {
    fn insert(&mut self, block: Block) -> anyhow::Result<()> {
        self.store.save_block(block)
    }

    fn get(&self, number: BlockNumber, block_id: HashValue) -> anyhow::Result<Option<Block>> {
        match self.store.get_dag_sync_block(number, block_id) {
            Ok(sync_block) => Ok(sync_block.block),
            Err(_e) => Ok(None),
        }
    }

    fn delete(&mut self, number: BlockNumber, block_id: HashValue) -> anyhow::Result<()> {
        self.store.delete_dag_sync_block(number, block_id)
    }

    fn delete_all(&mut self) -> anyhow::Result<()> {
        self.store.delete_all_dag_sync_block()
    }
}

impl<'a> SyncStoreIterator<'a> for SyncStoreAccessDB {
    type Type = Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>;

    fn iter(&'a self) -> Self::Type {
        let iter = self
            .store
            .iter_at_first()
            .expect("failed to get dag sync iterator");

        Box::new(iter.map(|result| {
            let (id_raw, data_raw) = result.expect("failed to get dag sync block");
            (id_raw, data_raw)
        }))
    }
}

impl SyncStoreAccess<'_> for SyncStoreAccessDB {}
