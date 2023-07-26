use rocksdb::WriteBatch;
use starcoin_storage::storage::InnerStore;

use crate::{db::DBStorage, errors::StoreError};

/// Abstraction over direct/batched DB writing
pub trait DbWriter {
    fn put(&mut self, cf_name: &str, key: &[u8], value: Vec<u8>) -> Result<(), StoreError>;
    fn delete(&mut self, cf_name: &str, key: &[u8]) -> Result<(), StoreError>;
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
    fn put(&mut self, cf_name: &str, key: &[u8], value: Vec<u8>) -> Result<(), StoreError> {
        self.db
            .put(cf_name, key.to_owned(), value)
            .map_err(|e| StoreError::DBIoError(e.to_string()))
    }

    fn delete(&mut self, cf_name: &str, key: &[u8]) -> Result<(), StoreError> {
        self.db
            .remove(cf_name, key.to_owned())
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
    fn put(&mut self, _cf_name: &str, key: &[u8], value: Vec<u8>) -> Result<(), StoreError> {
        self.batch.put(key, value);
        Ok(())
    }

    fn delete(&mut self, _cf_name: &str, key: &[u8]) -> Result<(), StoreError> {
        self.batch.delete(key);
        Ok(())
    }
}

impl<T: DbWriter> DbWriter for &mut T {
    #[inline]
    fn put(&mut self, cf_name: &str, key: &[u8], value: Vec<u8>) -> Result<(), StoreError> {
        (*self).put(cf_name, key, value)
    }

    #[inline]
    fn delete(&mut self, cf_name: &str, key: &[u8]) -> Result<(), StoreError> {
        (*self).delete(cf_name, key)
    }
}
