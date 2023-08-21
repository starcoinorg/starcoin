pub mod error;
pub mod schema;

use crate::error::StoreError;
use crate::schema::{KeyCodec, Schema, ValueCodec};
use parking_lot::Mutex;
use rocksdb::{DBIterator, IteratorMode, ReadOptions};
pub use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::InnerStore;
use std::collections::HashMap;
use std::sync::Arc;

pub type ColumnFamilyName = &'static str;

pub type WriteOp = starcoin_storage::storage::WriteOp<Vec<u8>, Vec<u8>>;

#[derive(Debug)]
pub struct SchemaBatch {
    rows: Mutex<HashMap<ColumnFamilyName, Vec<WriteOp>>>,
}

impl Default for SchemaBatch {
    fn default() -> Self {
        Self {
            rows: Mutex::new(HashMap::new()),
        }
    }
}

impl SchemaBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put<S: Schema>(&self, key: &S::Key, val: &S::Value) -> Result<(), StoreError> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let value = <S::Value as ValueCodec<S>>::encode_value(val)?;
        self.rows
            .lock()
            .entry(S::COLUMN_FAMILY)
            .or_insert_with(Vec::new)
            .push(WriteOp::Value(key, value));

        Ok(())
    }

    pub fn delete<S: Schema>(&self, key: &S::Key) -> Result<(), StoreError> {
        let key = <S::Key as KeyCodec<S>>::encode_key(key)?;

        self.rows
            .lock()
            .entry(S::COLUMN_FAMILY)
            .or_insert_with(Vec::new)
            .push(WriteOp::Deletion(key));

        Ok(())
    }
}

#[derive(Clone)]
pub struct DB {
    pub name: String, // for logging
    pub inner: Arc<DBStorage>,
}

impl DB {
    pub fn write_schemas(&self, batch: SchemaBatch) -> Result<(), StoreError> {
        let rows_locked = batch.rows.lock();

        for row in rows_locked.iter() {
            self.inner
                .write_batch_inner(row.0, row.1, false /*normal write*/)?
        }

        Ok(())
    }

    pub fn get<S: Schema>(&self, key: &S::Key) -> Result<Option<S::Value>, StoreError> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        self.inner
            .get(S::COLUMN_FAMILY, raw_key)?
            .map(|raw_value| <S::Value as ValueCodec<S>>::decode_value(&raw_value))
            .transpose()
    }

    pub fn put<S: Schema>(&self, key: &S::Key, value: &S::Value) -> Result<(), StoreError> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        let raw_val = <S::Value as ValueCodec<S>>::encode_value(value)?;

        self.inner.put(S::COLUMN_FAMILY, raw_key, raw_val)?;

        Ok(())
    }

    pub fn remove<S: Schema>(&self, key: &S::Key) -> Result<(), StoreError> {
        let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
        self.inner.remove(S::COLUMN_FAMILY, raw_key)?;
        Ok(())
    }

    pub fn flush_cf(&self, cf_name: &str) -> Result<(), StoreError> {
        Ok(self.inner.flush_cf(cf_name)?)
    }

    pub fn iterator_cf_opt<S: Schema>(
        &self,
        mode: IteratorMode,
        readopts: ReadOptions,
    ) -> Result<DBIterator, StoreError> {
        Ok(self
            .inner
            .raw_iterator_cf_opt(S::COLUMN_FAMILY, mode, readopts)?)
    }

    //fn get_cf_handle<S: Schema>(&self) -> Result<&rocksdb::ColumnFamily, StoreError> {
    //    Ok(self.inner.get_cf_handle(S::COLUMN_FAMILY)?)
    //}

    //pub fn get_pinned_cf<S: Schema>(
    //    &self,
    //    key: &S::Key,
    //) -> Result<Option<DBPinnableSlice>, StoreError> {
    //    let raw_key = <S::Key as KeyCodec<S>>::encode_key(key)?;
    //    let res = self
    //        .inner
    //        .raw_get_pinned_cf(S::COLUMN_FAMILY, raw_key.as_slice())?;

    //    Ok(res)
    //}
}
