pub(crate) use crate::schema::{
    transaction::Transaction,
    transaction_info::{OldTransactionInfo, TransactionInfo, TransactionInfoHash},
};
use anyhow::Result;
use rocksdb::ReadOptions;
use starcoin_schemadb::{db::DBStorage, iterator::SchemaIterator, schema::Schema, SchemaBatch};
use std::{marker::PhantomData, sync::Arc};

#[cfg(test)]
mod test;

pub(crate) type TransactionStorage = TransactionStore<Transaction>;
pub(crate) type TransactionInfoStorage = TransactionStore<TransactionInfo>;
pub(crate) type TransactionInfoHashStorage = TransactionStore<TransactionInfoHash>;
pub(crate) type OldTransactionInfoStorage = TransactionStore<OldTransactionInfo>;

#[derive(Clone)]
pub(crate) struct TransactionStore<S: Schema> {
    db: Arc<DBStorage>,
    _phantom: PhantomData<S>,
}

impl<S: Schema> TransactionStore<S> {
    pub fn new(db: &Arc<DBStorage>) -> Self {
        Self {
            db: Arc::clone(db),
            _phantom: Default::default(),
        }
    }

    pub fn get(&self, key: &S::Key) -> Result<Option<S::Value>> {
        self.db.get::<S>(key)
    }

    pub fn multi_get(&self, keys: &[S::Key]) -> Result<Vec<Option<S::Value>>> {
        self.db.multi_get::<S>(keys)
    }

    pub fn put(&self, key: &S::Key, value: &S::Value) -> Result<()> {
        self.db.put::<S>(key, value)
    }

    #[allow(unused)]
    pub fn put_all<'a>(
        &self,
        items: impl Iterator<Item = (&'a S::Key, &'a S::Value)>,
    ) -> Result<()> {
        let batch = SchemaBatch::new();
        for (key, value) in items {
            batch.put::<S>(key, value)?;
        }
        self.db.write_schemas(batch)
    }

    pub fn remove(&self, key: &S::Key) -> Result<()> {
        self.db.remove::<S>(key)
    }

    pub fn iter(&self) -> Result<SchemaIterator<S>> {
        self.db.iter(ReadOptions::default())
    }
}
