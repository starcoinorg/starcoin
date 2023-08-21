use super::cache::DagCache;

use itertools::Itertools;
use rocksdb::{Direction, IteratorMode, ReadOptions};
use starcoin_schemadb::{
    error::StoreError,
    schema::{KeyCodec, Schema, ValueCodec},
    DBStorage, SchemaBatch, DB,
};
use std::{
    collections::hash_map::RandomState, error::Error, hash::BuildHasher, marker::PhantomData,
    sync::Arc,
};

/// A concurrent DB store access with typed caching.
#[derive(Clone)]
pub struct CachedDbAccess<S: Schema, R = RandomState> {
    db: DB,
    // Cache
    cache: DagCache<S::Key, S::Value>,
    _phantom: PhantomData<R>,
}

impl<S: Schema, R> CachedDbAccess<S, R>
where
    R: BuildHasher + Default,
{
    pub fn new(db: Arc<DBStorage>, cache_size: u64) -> Self {
        Self {
            db: DB {
                name: "consensusdb".to_owned(),
                inner: Arc::clone(&db),
            },
            cache: DagCache::new_with_capacity(cache_size),
            _phantom: Default::default(),
        }
    }

    pub fn read_from_cache(&self, key: S::Key) -> Option<S::Value> {
        self.cache.get(&key)
    }

    pub fn has(&self, key: S::Key) -> Result<bool, StoreError> {
        Ok(self.cache.contains_key(&key) || self.db.get::<S>(&key)?.is_some())
    }

    pub fn read(&self, key: S::Key) -> Result<S::Value, StoreError> {
        if let Some(data) = self.cache.get(&key) {
            Ok(data)
        } else if let Some(data) = self.db.get::<S>(&key)? {
            self.cache.insert(key, data.clone());
            Ok(data)
        } else {
            Err(StoreError::KeyNotFound("".to_string()))
        }
    }

    pub fn iterator(
        &self,
    ) -> Result<impl Iterator<Item = Result<(Box<[u8]>, S::Value), Box<dyn Error>>> + '_, StoreError>
    {
        let db_iterator = self
            .db
            .iterator_cf_opt::<S>(IteratorMode::Start, ReadOptions::default())?;

        Ok(db_iterator.map(|iter_result| match iter_result {
            Ok((key, data_bytes)) => match S::Value::decode_value(&data_bytes) {
                Ok(data) => Ok((key, data)),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }))
    }

    pub fn write_batch(
        &self,
        batch: &mut SchemaBatch,
        key: S::Key,
        data: S::Value,
    ) -> Result<(), StoreError> {
        batch.put::<S>(&key, &data)?;
        self.cache.insert(key, data);
        Ok(())
    }

    pub fn write(&self, key: S::Key, data: S::Value) -> Result<(), StoreError> {
        self.db.put::<S>(&key, &data)?;
        self.cache.insert(key, data);
        Ok(())
    }

    pub fn write_many_batch(
        &self,
        batch: &mut SchemaBatch,
        iter: &mut (impl Iterator<Item = (S::Key, S::Value)> + Clone),
    ) -> Result<(), StoreError> {
        for (key, data) in iter {
            batch.put::<S>(&key, &data)?;
            self.cache.insert(key, data);
        }
        Ok(())
    }

    /// Write directly from an iterator and do not cache any data. NOTE: this action also clears the cache
    pub fn write_many_without_cache(
        &self,
        iter: &mut impl Iterator<Item = (S::Key, S::Value)>,
    ) -> Result<(), StoreError> {
        let mut batch = SchemaBatch::new();
        for (key, data) in iter {
            batch.put::<S>(&key, &data)?;
        }
        self.db.write_schemas(batch)?;

        // The cache must be cleared in order to avoid invalidated entries
        self.cache.remove_all();
        Ok(())
    }

    pub fn remove(&self, key: &S::Key) -> Result<(), StoreError> {
        self.cache.remove(&key);
        self.db.remove::<S>(&key)?;
        Ok(())
    }

    pub fn remove_batch(&self, batch: &mut SchemaBatch, key: &S::Key) -> Result<(), StoreError> {
        self.cache.remove(key);
        batch.delete::<S>(key)?;
        Ok(())
    }

    pub fn delete_many(
        &self,
        key_iter: &mut (impl Iterator<Item = S::Key> + Clone),
    ) -> Result<(), StoreError> {
        let key_iter_clone = key_iter.clone();
        self.cache.remove_many(key_iter);
        let batch = SchemaBatch::new();
        for key in key_iter_clone {
            batch.delete::<S>(&key)?;
        }
        self.db.write_schemas(batch)?;
        Ok(())
    }

    pub fn delete_all(&self) -> Result<(), StoreError> {
        self.cache.remove_all();
        let keys = self
            .db
            .iterator_cf_opt::<S>(IteratorMode::Start, ReadOptions::default())?
            .map(|iter_result| match iter_result {
                Ok((key, _)) => Ok::<_, rocksdb::Error>(key),
                Err(e) => Err(e),
            })
            .collect_vec();
        let batch = SchemaBatch::new();
        for key in keys {
            batch.delete::<S>(&S::Key::decode_key(&key?)?)?;
        }
        self.db.write_schemas(batch)?;
        Ok(())
    }

    /// A dynamic iterator that can iterate through a specific prefix, and from a certain start point.
    //TODO: loop and chain iterators for multi-prefix iterator.
    pub fn seek_iterator(
        &self,
        seek_from: Option<S::Key>, // iter whole range if None
        limit: usize,              // amount to take.
        skip_first: bool, // skips the first value, (useful in conjunction with the seek-key, as to not re-retrieve).
    ) -> Result<impl Iterator<Item = Result<(Box<[u8]>, S::Value), Box<dyn Error>>> + '_, StoreError>
    {
        let read_opts = ReadOptions::default();
        let mut db_iterator = match seek_from {
            Some(seek_key) => self.db.iterator_cf_opt::<S>(
                IteratorMode::From(seek_key.encode_key()?.as_slice(), Direction::Forward),
                read_opts,
            ),
            None => self.db.iterator_cf_opt::<S>(IteratorMode::Start, read_opts),
        }?;

        if skip_first {
            db_iterator.next();
        }

        Ok(db_iterator.take(limit).map(move |item| match item {
            Ok((key_bytes, value_bytes)) => match S::Value::decode_value(value_bytes.as_ref()) {
                Ok(value) => Ok((key_bytes, value)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }))
    }
}
