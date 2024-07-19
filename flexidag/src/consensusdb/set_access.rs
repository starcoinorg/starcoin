use crate::consensusdb::{cache::DagCache, error::StoreError, schema::Schema, writer::DbWriter};
use parking_lot::RwLock;
use rocksdb::{IteratorMode, ReadOptions};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use starcoin_storage::db_storage::DBStorage;
use std::error::Error;
use std::{collections::hash_map::RandomState, marker::PhantomData, sync::Arc};

#[derive(Clone)]
pub struct DbSetAccess<TKey, TData, R = RandomState> {
    db: Arc<DBStorage>,
    cf: &'static str,
    _phantom: PhantomData<(TKey, TData, R)>,
}

impl<TKey, TData> DbSetAccess<TKey, TData> {
    pub fn new(db: Arc<DBStorage>, cf: &'static str) -> Self {
        Self {
            db,
            cf,
            _phantom: Default::default(),
        }
    }

    pub fn read(&self, key: TKey) -> Result<Vec<TData>, StoreError>
    where
        TKey: AsRef<[u8]>,
        TData: DeserializeOwned,
    {
        self.seek_iterator(key, usize::MAX, false)
            .map(|iter| {
                iter.filter_map(Result::ok)
                    .map(|r| bcs_ext::from_bytes::<TData>(r.as_ref()).unwrap())
                    .collect::<Vec<_>>()
            })
            .map_err(Into::into)
    }

    pub fn write(
        &self,
        mut writer: impl DbWriter,
        key: TKey,
        value: TData,
    ) -> Result<(), StoreError>
    where
        TKey: AsRef<[u8]>,
        TData: Serialize,
    {
        let db_key = key
            .as_ref()
            .iter()
            .chain(bcs_ext::to_bytes(&value).unwrap().iter())
            .copied()
            .collect::<Vec<_>>();

        writer.put_inner(&db_key, &[], self.cf)
    }

    fn seek_iterator(
        &self,
        db_key: TKey,
        limit: usize,     // amount to take.
        skip_first: bool, // skips the first value, (useful in conjunction with the seek-key, as to not re-retrieve).
    ) -> Result<impl Iterator<Item = Result<Box<[u8]>, Box<dyn Error>>> + '_, StoreError>
    where
        TKey: AsRef<[u8]>,
    {
        let key_len = db_key.as_ref().len();
        let mut read_opts = ReadOptions::default();
        read_opts.set_iterate_range(rocksdb::PrefixRange::<&[u8]>(db_key.as_ref()));

        let mut db_iterator = self
            .db
            .raw_iterator_cf_opt(self.cf, IteratorMode::Start, read_opts)
            .map_err(|e| StoreError::CFNotExist(e.to_string()))?;

        if skip_first {
            db_iterator.next();
        }

        Ok(db_iterator.take(limit).map(move |item| match item {
            Ok((key_bytes, _)) => Ok(key_bytes[key_len..].into()),
            Err(err) => Err(err.into()),
        }))
    }
}

#[derive(Clone)]
pub struct CachedDbSetAccess<S: Schema, R = RandomState> {
    inner: DbSetAccess<S::Key, S::Value, R>,
    cache: DagCache<S::Key, Arc<RwLock<Vec<S::Value>>>>,
}
impl<S: Schema> CachedDbSetAccess<S> {
    pub fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            inner: DbSetAccess::new(db, S::COLUMN_FAMILY),
            cache: DagCache::new_with_capacity(cache_size),
        }
    }

    // Mark the key has been initialized in memory to speed up the read operation.
    pub fn initialize(&self, key: S::Key) {
        self.cache.insert(key, Arc::new(RwLock::new(Vec::new())));
    }

    pub fn read(&self, key: S::Key) -> Result<Arc<RwLock<Vec<S::Value>>>, StoreError>
    where
        S::Key: AsRef<[u8]>,
        S::Value: DeserializeOwned,
    {
        self.cache.get(&key).map_or_else(
            || {
                self.inner.read(key.clone()).map(|v| {
                    let v = Arc::new(RwLock::new(v));
                    self.cache.insert(key, v.clone());
                    v
                })
            },
            Ok,
        )
    }

    pub fn write(
        &self,
        writer: impl DbWriter,
        key: S::Key,
        value: S::Value,
    ) -> Result<(), StoreError>
    where
        S::Value: std::cmp::PartialEq + Serialize + DeserializeOwned,
        S::Key: AsRef<[u8]>,
    {
        let data = self.read(key.clone())?;
        let mut data_writer = data.write();
        // acquire exclusive write lock before checking the existence of the value
        if !data_writer.contains(&value) {
            self.inner.write(writer, key, value.clone())?;
            data_writer.push(value);
        }
        Ok(())
    }
}
