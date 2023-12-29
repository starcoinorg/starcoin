use rocksdb::WriteBatch;
use starcoin_storage::storage::InnerStore;

use super::schema::{KeyCodec, Schema, ValueCodec};
use super::{db::DBStorage, error::StoreError};

/// Abstraction over direct/batched DB writing
pub trait DbWriter {
    fn put<S: Schema>(&mut self, key: &S::Key, value: &S::Value) -> Result<(), StoreError>;
    fn delete<S: Schema>(&mut self, key: &S::Key) -> Result<(), StoreError>;
}

pub struct DirectDbWriter<'a> {
    db: &'a DBStorage,
}

impl<'a> DirectDbWriter<'a> {
    pub fn new(db: &'a DBStorage) -> Self {
        Self { db }
    }
}

impl DbWriter for DirectDbWriter<'_> {
    fn put<S: Schema>(&mut self, key: &S::Key, value: &S::Value) -> Result<(), StoreError> {
        let bin_key = key.encode_key()?;
        let bin_data = value.encode_value()?;
        self.db
            .put(S::COLUMN_FAMILY, bin_key, bin_data)
            .map_err(|e| StoreError::DBIoError(e.to_string()))
    }

    fn delete<S: Schema>(&mut self, key: &S::Key) -> Result<(), StoreError> {
        let key = key.encode_key()?;
        self.db
            .remove(S::COLUMN_FAMILY, key)
            .map_err(|e| StoreError::DBIoError(e.to_string()))
    }
}

pub struct BatchDbWriter<'a> {
    batch: &'a mut WriteBatch,
}

impl<'a> BatchDbWriter<'a> {
    pub fn new(batch: &'a mut WriteBatch) -> Self {
        Self { batch }
    }
}

impl DbWriter for BatchDbWriter<'_> {
    fn put<S: Schema>(&mut self, key: &S::Key, value: &S::Value) -> Result<(), StoreError> {
        let key = key.encode_key()?;
        let value = value.encode_value()?;
        self.batch.put(key, value);
        Ok(())
    }

    fn delete<S: Schema>(&mut self, key: &S::Key) -> Result<(), StoreError> {
        let key = key.encode_key()?;
        self.batch.delete(key);
        Ok(())
    }
}

impl<T: DbWriter> DbWriter for &mut T {
    #[inline]
    fn put<S: Schema>(&mut self, key: &S::Key, value: &S::Value) -> Result<(), StoreError> {
        (*self).put::<S>(key, value)
    }

    #[inline]
    fn delete<S: Schema>(&mut self, key: &S::Key) -> Result<(), StoreError> {
        (*self).delete::<S>(key)
    }
}
