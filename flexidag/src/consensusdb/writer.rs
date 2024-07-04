use rocksdb::WriteBatch;
use starcoin_storage::storage::InnerStore;

use super::schema::{KeyCodec, Schema, ValueCodec};
use super::{db::DBStorage, error::StoreError};

/// Abstraction over direct/batched DB writing
pub trait DbWriter {
    fn put<S: Schema>(&mut self, key: &S::Key, value: &S::Value) -> Result<(), StoreError>;
    fn delete<S: Schema>(&mut self, key: &S::Key) -> Result<(), StoreError>;
    fn put_inner<S: Schema>(
        &mut self,
        key: &dyn AsRef<[u8]>,
        value: &dyn AsRef<[u8]>,
    ) -> Result<(), StoreError>;
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

    fn put_inner<S: Schema>(
        &mut self,
        key: &dyn AsRef<[u8]>,
        value: &dyn AsRef<[u8]>,
    ) -> Result<(), StoreError> {
        self.db
            .put(
                S::COLUMN_FAMILY,
                key.as_ref().to_vec(),
                value.as_ref().to_vec(),
            )
            .map_err(|e| StoreError::DBIoError(e.to_string()))
    }
}

pub struct BatchDbWriter<'a> {
    batch: &'a mut WriteBatch,
    db: &'a DBStorage,
}

impl<'a> BatchDbWriter<'a> {
    pub fn new(batch: &'a mut WriteBatch, db: &'a DBStorage) -> Self {
        Self { batch, db }
    }
}

impl DbWriter for BatchDbWriter<'_> {
    fn put<S: Schema>(&mut self, key: &S::Key, value: &S::Value) -> Result<(), StoreError> {
        let key = key.encode_key()?;
        let value = value.encode_value()?;
        let cf_handle = self.db.get_cf_handle(S::COLUMN_FAMILY);
        self.batch.put_cf(cf_handle, key, value);
        Ok(())
    }

    fn delete<S: Schema>(&mut self, key: &S::Key) -> Result<(), StoreError> {
        let key = key.encode_key()?;
        let cf_handle = self.db.get_cf_handle(S::COLUMN_FAMILY);
        self.batch.delete_cf(cf_handle, key);
        Ok(())
    }

    fn put_inner<S: Schema>(
        &mut self,
        key: &dyn AsRef<[u8]>,
        value: &dyn AsRef<[u8]>,
    ) -> Result<(), StoreError> {
        self.batch.put(key, value);
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

    #[inline]
    fn put_inner<S: Schema>(
        &mut self,
        key: &dyn AsRef<[u8]>,
        value: &dyn AsRef<[u8]>,
    ) -> Result<(), StoreError> {
        (*self).put_inner::<S>(key, value)
    }
}
