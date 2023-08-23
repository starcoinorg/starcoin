use parking_lot::RwLock;
use starcoin_schemadb::{
    error::StoreError,
    schema::{KeyCodec, Schema},
    DBStorage, SchemaBatch, DB,
};
use std::sync::Arc;

/// A cached DB item with concurrency support
#[derive(Clone)]
pub struct CachedDbItem<S: Schema> {
    db: DB,
    key: S::Key,
    cached_item: Arc<RwLock<Option<S::Value>>>,
}

impl<S: Schema> CachedDbItem<S> {
    pub fn new(db: Arc<DBStorage>, key: S::Key) -> Self {
        Self {
            db: DB {
                name: "cacheitem".to_owned(),
                inner: Arc::clone(&db),
            },
            key,
            cached_item: Arc::new(RwLock::new(None)),
        }
    }

    pub fn read(&self) -> Result<S::Value, StoreError> {
        if let Some(item) = self.cached_item.read().clone() {
            return Ok(item);
        }
        if let Some(item) = self.db.get::<S>(&self.key)? {
            *self.cached_item.write() = Some(item.clone());
            Ok(item)
        } else {
            Err(StoreError::KeyNotFound(
                String::from_utf8(self.key.encode_key()?)
                    .unwrap_or_else(|_| ("unrecoverable key string").to_string()),
            ))
        }
    }

    pub fn write_batch(
        &mut self,
        batch: &mut SchemaBatch,
        item: &S::Value,
    ) -> Result<(), StoreError> {
        *self.cached_item.write() = Some(item.clone());
        batch.put::<S>(&self.key, item)?;
        Ok(())
    }

    pub fn write(&mut self, item: &S::Value) -> Result<(), StoreError> {
        *self.cached_item.write() = Some(item.clone());
        self.db.put::<S>(&self.key, item)?;
        Ok(())
    }

    pub fn remove_batch(&mut self, batch: &mut SchemaBatch) -> Result<(), StoreError>
where {
        *self.cached_item.write() = None;
        batch.delete::<S>(&self.key)?;
        Ok(())
    }

    pub fn remove(&mut self) -> Result<(), StoreError> {
        *self.cached_item.write() = None;
        self.db.remove::<S>(&self.key)
    }

    pub fn update<F>(&mut self, op: F) -> Result<S::Value, StoreError>
    where
        F: Fn(S::Value) -> S::Value,
    {
        let mut guard = self.cached_item.write();
        let mut item = if let Some(item) = guard.take() {
            item
        } else if let Some(item) = self.db.get::<S>(&self.key)? {
            item
        } else {
            return Err(StoreError::KeyNotFound("".to_string()));
        };

        item = op(item); // Apply the update op
        *guard = Some(item.clone());
        self.db.put::<S>(&self.key, &item)?;
        Ok(item)
    }
}
