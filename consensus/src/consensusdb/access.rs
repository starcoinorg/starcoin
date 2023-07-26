use crate::{cache::DagCache, db::DBStorage, errors::StoreError};

use super::prelude::{Cache, DbWriter};
use itertools::Itertools;
use rocksdb::{Direction, IteratorMode, ReadOptions};
use serde::{de::DeserializeOwned, Serialize};
use starcoin_storage::storage::RawDBStorage;
use std::{
    collections::hash_map::RandomState, error::Error, hash::BuildHasher, marker::PhantomData,
    sync::Arc,
};

/// A concurrent DB store access with typed caching.
#[derive(Clone)]
pub struct CachedDbAccess<TKey, TData, S = RandomState>
where
    TKey: Clone + std::hash::Hash + Eq + Send + Sync + AsRef<[u8]>,
    TData: Clone + Send + Sync + DeserializeOwned,
{
    db: Arc<DBStorage>,

    // Cache
    cache: Cache<TKey>,

    // DB bucket/path
    prefix: &'static str,

    _phantom: PhantomData<(TData, S)>,
}

impl<TKey, TData, S> CachedDbAccess<TKey, TData, S>
where
    TKey: Clone + std::hash::Hash + Eq + Send + Sync + AsRef<[u8]>,
    TData: Clone + Send + Sync + DeserializeOwned,
    S: BuildHasher + Default,
{
    pub fn new(db: Arc<DBStorage>, cache_size: u64, prefix: &'static str) -> Self {
        Self {
            db,
            cache: Cache::new_with_capacity(cache_size),
            prefix,
            _phantom: Default::default(),
        }
    }

    pub fn read_from_cache(&self, key: TKey) -> Result<Option<TData>, StoreError>
    where
        TKey: Copy + AsRef<[u8]>,
    {
        self.cache
            .get(&key)
            .map(|b| bincode::deserialize(&b).map_err(StoreError::DeserializationError))
            .transpose()
    }

    pub fn has(&self, key: TKey) -> Result<bool, StoreError>
    where
        TKey: Clone + AsRef<[u8]>,
    {
        Ok(self.cache.contains_key(&key)
            || self
                .db
                .raw_get_pinned_cf(self.prefix, key)
                .map_err(|_| StoreError::CFNotExist(self.prefix.to_string()))?
                .is_some())
    }

    pub fn read(&self, key: TKey) -> Result<TData, StoreError>
    where
        TKey: Clone + AsRef<[u8]> + ToString,
        TData: DeserializeOwned, // We need `DeserializeOwned` since the slice coming from `db.get_pinned_cf` has short lifetime
    {
        if let Some(data) = self.cache.get(&key) {
            let data = bincode::deserialize(&data)?;
            Ok(data)
        } else if let Some(slice) = self
            .db
            .raw_get_pinned_cf(self.prefix, &key)
            .map_err(|_| StoreError::CFNotExist(self.prefix.to_string()))?
        {
            let data: TData = bincode::deserialize(&slice)?;
            self.cache.insert(key, slice.to_vec());
            Ok(data)
        } else {
            Err(StoreError::KeyNotFound(key.to_string()))
        }
    }

    pub fn iterator(
        &self,
    ) -> Result<impl Iterator<Item = Result<(Box<[u8]>, TData), Box<dyn Error>>> + '_, StoreError>
    where
        TKey: Clone + AsRef<[u8]>,
        TData: DeserializeOwned, // We need `DeserializeOwned` since the slice coming from `db.get_pinned_cf` has short lifetime
    {
        let db_iterator = self
            .db
            .raw_iterator_cf_opt(self.prefix, IteratorMode::Start, ReadOptions::default())
            .map_err(|e| StoreError::CFNotExist(e.to_string()))?;

        Ok(db_iterator.map(|iter_result| match iter_result {
            Ok((key, data_bytes)) => match bincode::deserialize(&data_bytes) {
                Ok(data) => Ok((key, data)),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }))
    }

    pub fn write(&self, mut writer: impl DbWriter, key: TKey, data: TData) -> Result<(), StoreError>
    where
        TKey: Clone + AsRef<[u8]>,
        TData: Serialize,
    {
        let bin_data = bincode::serialize(&data)?;
        self.cache.insert(key.clone(), bin_data.clone());
        writer.put(self.prefix, key.as_ref(), bin_data)?;
        Ok(())
    }

    pub fn write_many(
        &self,
        mut writer: impl DbWriter,
        iter: &mut (impl Iterator<Item = (TKey, TData)> + Clone),
    ) -> Result<(), StoreError>
    where
        TKey: Clone + AsRef<[u8]>,
        TData: Serialize,
    {
        for (key, data) in iter {
            let bin_data = bincode::serialize(&data)?;
            self.cache.insert(key.clone(), bin_data.clone());
            writer.put(self.prefix, key.as_ref(), bin_data)?;
        }
        Ok(())
    }

    /// Write directly from an iterator and do not cache any data. NOTE: this action also clears the cache
    pub fn write_many_without_cache(
        &self,
        mut writer: impl DbWriter,
        iter: &mut impl Iterator<Item = (TKey, TData)>,
    ) -> Result<(), StoreError>
    where
        TKey: Clone + AsRef<[u8]>,
        TData: Serialize,
    {
        for (key, data) in iter {
            let bin_data = bincode::serialize(&data)?;
            writer.put(self.prefix, key.as_ref(), bin_data)?;
        }
        // The cache must be cleared in order to avoid invalidated entries
        self.cache.remove_all();
        Ok(())
    }

    pub fn delete(&self, mut writer: impl DbWriter, key: TKey) -> Result<(), StoreError>
    where
        TKey: Clone + AsRef<[u8]>,
    {
        self.cache.remove(&key);
        writer.delete(self.prefix, key.as_ref())?;
        Ok(())
    }

    pub fn delete_many(
        &self,
        mut writer: impl DbWriter,
        key_iter: &mut (impl Iterator<Item = TKey> + Clone),
    ) -> Result<(), StoreError>
    where
        TKey: Clone + AsRef<[u8]>,
    {
        let key_iter_clone = key_iter.clone();
        self.cache.remove_many(key_iter);
        for key in key_iter_clone {
            writer.delete(self.prefix, key.as_ref())?;
        }
        Ok(())
    }

    pub fn delete_all(&self, mut writer: impl DbWriter) -> Result<(), StoreError>
    where
        TKey: Clone + AsRef<[u8]>,
    {
        self.cache.remove_all();
        let keys = self
            .db
            .raw_iterator_cf_opt(self.prefix, IteratorMode::Start, ReadOptions::default())
            .map_err(|e| StoreError::CFNotExist(e.to_string()))?
            .map(|iter_result| match iter_result {
                Ok((key, _)) => Ok::<_, rocksdb::Error>(key),
                Err(e) => Err(e),
            })
            .collect_vec();
        for key in keys {
            writer.delete(self.prefix, key?.as_ref())?;
        }
        Ok(())
    }

    /// A dynamic iterator that can iterate through a specific prefix, and from a certain start point.
    //TODO: loop and chain iterators for multi-prefix iterator.
    pub fn seek_iterator(
        &self,
        seek_from: Option<TKey>, // iter whole range if None
        limit: usize,            // amount to take.
        skip_first: bool, // skips the first value, (useful in conjunction with the seek-key, as to not re-retrieve).
    ) -> Result<impl Iterator<Item = Result<(Box<[u8]>, TData), Box<dyn Error>>> + '_, StoreError>
    where
        TKey: Clone + AsRef<[u8]>,
        TData: DeserializeOwned,
    {
        let read_opts = ReadOptions::default();
        let mut db_iterator = match seek_from {
            Some(seek_key) => self.db.raw_iterator_cf_opt(
                self.prefix,
                IteratorMode::From(seek_key.as_ref(), Direction::Forward),
                read_opts,
            ),
            None => self
                .db
                .raw_iterator_cf_opt(self.prefix, IteratorMode::Start, read_opts),
        }
        .map_err(|e| StoreError::CFNotExist(e.to_string()))?;

        if skip_first {
            db_iterator.next();
        }

        Ok(db_iterator.take(limit).map(move |item| match item {
            Ok((key_bytes, value_bytes)) => {
                match bincode::deserialize::<TData>(value_bytes.as_ref()) {
                    Ok(value) => Ok((key_bytes, value)),
                    Err(err) => Err(err.into()),
                }
            }
            Err(err) => Err(err.into()),
        }))
    }
}
