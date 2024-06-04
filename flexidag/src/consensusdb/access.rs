use super::{cache::DagCache, db::DBStorage, error::StoreError};

use super::prelude::DbWriter;
use super::schema::{KeyCodec, Schema, ValueCodec};
use itertools::Itertools;
use rocksdb::{Direction, IteratorMode, ReadOptions};
use starcoin_storage::storage::RawDBStorage;
use std::{
    collections::hash_map::RandomState, error::Error, hash::BuildHasher, marker::PhantomData,
    sync::Arc,
};

/// A concurrent DB store access with typed caching.
#[derive(Clone)]
pub struct CachedDbAccess<S: Schema, R = RandomState> {
    db: Arc<DBStorage>,

    // Cache
    cache: DagCache<S::Key, S::Value>,

    _phantom: PhantomData<R>,
}

impl<S: Schema, R> CachedDbAccess<S, R>
where
    R: BuildHasher + Default,
{
    pub fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            db,
            cache: DagCache::new_with_capacity(cache_size),
            _phantom: Default::default(),
        }
    }

    pub fn read_from_cache(&self, key: S::Key) -> Option<S::Value> {
        self.cache.get(&key)
    }

    pub fn has(&self, key: S::Key) -> Result<bool, StoreError> {
        Ok(self.cache.contains_key(&key)
            || self
                .db
                .raw_get_pinned_cf(S::COLUMN_FAMILY, key.encode_key().unwrap())
                .map_err(|_| StoreError::CFNotExist(S::COLUMN_FAMILY.to_string()))?
                .is_some())
    }

    pub fn read(&self, key: S::Key) -> Result<S::Value, StoreError> {
        if let Some(data) = self.cache.get(&key) {
            Ok(data)
        } else if let Some(slice) = self
            .db
            .raw_get_pinned_cf(S::COLUMN_FAMILY, key.encode_key().unwrap())
            .map_err(|_| StoreError::CFNotExist(S::COLUMN_FAMILY.to_string()))?
        {
            let data = S::Value::decode_value(slice.as_ref())
                .map_err(|o| StoreError::DecodeError(o.to_string()))?;
            self.cache.insert(key, data.clone());
            Ok(data)
        } else {
            Err(StoreError::KeyNotFound(format!("{:?}", key)))
        }
    }

    pub fn iterator(
        &self,
    ) -> Result<impl Iterator<Item = Result<(S::Key, S::Value), Box<dyn Error>>> + '_, StoreError>
    {
        let db_iterator = self
            .db
            .raw_iterator_cf_opt(
                S::COLUMN_FAMILY,
                IteratorMode::Start,
                ReadOptions::default(),
            )
            .map_err(|e| StoreError::CFNotExist(e.to_string()))?;

        Ok(db_iterator.map(|iter_result| match iter_result {
            Ok((key, data_bytes)) => match (
                S::Key::decode_key(&key),
                S::Value::decode_value(&data_bytes),
            ) {
                (Ok(key), Ok(data)) => Ok((key, data)),
                (Err(e), _) => Err(e.into()),
                (_, Err(e)) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }))
    }

    pub fn write(
        &self,
        mut writer: impl DbWriter,
        key: S::Key,
        data: S::Value,
    ) -> Result<(), StoreError> {
        writer.put::<S>(&key, &data)?;
        self.cache.insert(key, data);
        Ok(())
    }

    pub fn write_many(
        &self,
        mut writer: impl DbWriter,
        iter: &mut (impl Iterator<Item = (S::Key, S::Value)> + Clone),
    ) -> Result<(), StoreError> {
        for (key, data) in iter {
            writer.put::<S>(&key, &data)?;
            self.cache.insert(key, data);
        }
        Ok(())
    }

    /// Write directly from an iterator and do not cache any data. NOTE: this action also clears the cache
    pub fn write_many_without_cache(
        &self,
        mut writer: impl DbWriter,
        iter: &mut impl Iterator<Item = (S::Key, S::Value)>,
    ) -> Result<(), StoreError> {
        for (key, data) in iter {
            writer.put::<S>(&key, &data)?;
        }
        // The cache must be cleared in order to avoid invalidated entries
        self.cache.remove_all();
        Ok(())
    }

    pub fn delete(&self, mut writer: impl DbWriter, key: S::Key) -> Result<(), StoreError> {
        self.cache.remove(&key);
        writer.delete::<S>(&key)?;
        Ok(())
    }

    pub fn delete_many(
        &self,
        mut writer: impl DbWriter,
        key_iter: &mut (impl Iterator<Item = S::Key> + Clone),
    ) -> Result<(), StoreError> {
        let key_iter_clone = key_iter.clone();
        self.cache.remove_many(key_iter);
        for key in key_iter_clone {
            writer.delete::<S>(&key)?;
        }
        Ok(())
    }

    pub fn delete_all(&self, mut writer: impl DbWriter) -> Result<(), StoreError> {
        self.cache.remove_all();
        let keys = self
            .db
            .raw_iterator_cf_opt(
                S::COLUMN_FAMILY,
                IteratorMode::Start,
                ReadOptions::default(),
            )
            .map_err(|e| StoreError::CFNotExist(e.to_string()))?
            .map(|iter_result| match iter_result {
                Ok((key, _)) => Ok::<_, rocksdb::Error>(key),
                Err(e) => Err(e),
            })
            .collect_vec();
        for key in keys {
            writer.delete::<S>(&S::Key::decode_key(&key?)?)?;
        }
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
            Some(seek_key) => self.db.raw_iterator_cf_opt(
                S::COLUMN_FAMILY,
                IteratorMode::From(seek_key.encode_key()?.as_slice(), Direction::Forward),
                read_opts,
            ),
            None => self
                .db
                .raw_iterator_cf_opt(S::COLUMN_FAMILY, IteratorMode::Start, read_opts),
        }
        .map_err(|e| StoreError::CFNotExist(e.to_string()))?;

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
