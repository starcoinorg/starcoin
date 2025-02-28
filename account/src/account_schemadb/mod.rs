use anyhow::Result;
use starcoin_schemadb::schema::Schema;
use starcoin_storage::cache_storage::GCacheStorage;
use std::sync::Arc;

mod accepted_token;
mod global_setting;
mod private_key;
mod public_key;
mod setting;

pub(crate) use accepted_token::*;
pub(crate) use global_setting::*;
pub(crate) use private_key::*;
pub(crate) use public_key::*;
pub(crate) use setting::*;
use starcoin_schemadb::{db::DBStorage as DB, SchemaBatch};

#[derive(Clone)]
pub(super) struct AccountStore<S: Schema> {
    cache: Arc<GCacheStorage<S::Key, S::Value>>,
    db: Option<Arc<DB>>,
}

impl<S: Schema> AccountStore<S> {
    // create an memory-based store
    pub fn new() -> Self {
        Self {
            cache: Arc::new(GCacheStorage::<S::Key, S::Value>::new(None)),
            db: None,
        }
    }
    pub fn new_with_db(db: &Arc<DB>) -> Self {
        Self {
            cache: Arc::new(GCacheStorage::<S::Key, S::Value>::new(None)),
            db: Some(Arc::clone(db)),
        }
    }

    pub fn get(&self, key: &S::Key) -> Result<Option<S::Value>> {
        self.cache
            .get_inner(key)
            .map(|val| Ok(Some(val)))
            .unwrap_or_else(|| {
                self.db
                    .as_ref()
                    .map_or_else(|| Ok(None), |db| db.get::<S>(key))
            })
    }

    pub fn put(&self, key: S::Key, value: S::Value) -> Result<()> {
        self.db
            .as_ref()
            .map_or_else(|| Ok(()), |db| db.put::<S>(&key, &value))
            .map(|_| {
                self.cache.put_inner(key, value);
            })
    }

    pub fn remove(&self, key: &S::Key) -> Result<()> {
        self.db
            .as_ref()
            .map_or_else(|| Ok(()), |db| db.remove::<S>(key))
            .map(|_| {
                self.cache.remove_inner(key);
            })
    }

    pub fn put_batch(&self, key: S::Key, value: S::Value, batch: &SchemaBatch) -> Result<()> {
        batch.put::<S>(&key, &value)?;
        self.put(key, value)
    }

    pub fn remove_batch(&self, key: &S::Key, batch: &SchemaBatch) -> Result<()> {
        batch.delete::<S>(key)?;
        self.remove(key)
    }
}
