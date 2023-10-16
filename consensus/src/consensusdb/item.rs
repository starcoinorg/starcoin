use super::prelude::DbWriter;
use super::schema::{KeyCodec, Schema, ValueCodec};
use super::{db::DBStorage, error::StoreError};
use parking_lot::RwLock;
use starcoin_storage::storage::RawDBStorage;
use std::sync::Arc;

/// A cached DB item with concurrency support
#[derive(Clone)]
pub struct CachedDbItem<S: Schema> {
    db: Arc<DBStorage>,
    key: S::Key,
    cached_item: Arc<RwLock<Option<S::Value>>>,
}

impl<S: Schema> CachedDbItem<S> {
    pub fn new(db: Arc<DBStorage>, key: S::Key) -> Self {
        Self {
            db,
            key,
            cached_item: Arc::new(RwLock::new(None)),
        }
    }

    pub fn read(&self) -> Result<S::Value, StoreError> {
        if let Some(item) = self.cached_item.read().clone() {
            return Ok(item);
        }
        if let Some(slice) = self
            .db
            .raw_get_pinned_cf(S::COLUMN_FAMILY, &self.key.encode_key()?)
            .map_err(|_| StoreError::CFNotExist(S::COLUMN_FAMILY.to_string()))?
        {
            let item = S::Value::decode_value(&slice)?;
            *self.cached_item.write() = Some(item.clone());
            Ok(item)
        } else {
            Err(StoreError::KeyNotFound(
                String::from_utf8(self.key.encode_key()?)
                    .unwrap_or(("unrecoverable key string").to_string()),
            ))
        }
    }

    pub fn write(&mut self, mut writer: impl DbWriter, item: &S::Value) -> Result<(), StoreError> {
        *self.cached_item.write() = Some(item.clone());
        writer.put::<S>(&self.key, item)?;
        Ok(())
    }

    pub fn remove(&mut self, mut writer: impl DbWriter) -> Result<(), StoreError>
where {
        *self.cached_item.write() = None;
        writer.delete::<S>(&self.key)?;
        Ok(())
    }

    pub fn update<F>(&mut self, mut writer: impl DbWriter, op: F) -> Result<S::Value, StoreError>
    where
        F: Fn(S::Value) -> S::Value,
    {
        let mut guard = self.cached_item.write();
        let mut item = if let Some(item) = guard.take() {
            item
        } else if let Some(slice) = self
            .db
            .raw_get_pinned_cf(S::COLUMN_FAMILY, &self.key.encode_key()?)
            .map_err(|_| StoreError::CFNotExist(S::COLUMN_FAMILY.to_string()))?
        {
            let item = S::Value::decode_value(&slice)?;
            item
        } else {
            return Err(StoreError::KeyNotFound("".to_string()));
        };

        item = op(item); // Apply the update op
        *guard = Some(item.clone());
        writer.put::<S>(&self.key, &item)?;
        Ok(item)
    }
}
