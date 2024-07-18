use crate::consensusdb::schema::{KeyCodec, ValueCodec};
use crate::consensusdb::{cache::DagCache, error::StoreError, schema::Schema, writer::DbWriter};
use parking_lot::RwLock;
use rocksdb::{IteratorMode, ReadOptions};
use starcoin_storage::db_storage::DBStorage;
use std::error::Error;
use std::{collections::hash_map::RandomState, marker::PhantomData, sync::Arc};

#[derive(Clone)]
pub struct DbSetAccess<S: Schema, R = RandomState> {
    db: Arc<DBStorage>,
    _phantom: PhantomData<(S, R)>,
}

impl<S: Schema> DbSetAccess<S> {
    pub fn new(db: Arc<DBStorage>) -> Self {
        Self {
            db,
            _phantom: Default::default(),
        }
    }

    pub fn read(&self, key: S::Key) -> Result<Vec<S::Value>, StoreError> {
        self.seek_iterator(key, usize::MAX, false)
            .map(|iter| iter.filter_map(Result::ok).collect::<Vec<S::Value>>())
            .map_err(Into::into)
    }

    pub fn write(
        &self,
        mut writer: impl DbWriter,
        key: S::Key,
        value: S::Value,
    ) -> Result<(), StoreError> {
        let db_key = key
            .encode_key()?
            .iter()
            .chain(value.encode_value()?.iter())
            .copied()
            .collect::<Vec<_>>();

        writer.put_inner::<S>(&db_key, &[])
    }

    fn seek_iterator(
        &self,
        db_key: S::Key,
        limit: usize,     // amount to take.
        skip_first: bool, // skips the first value, (useful in conjunction with the seek-key, as to not re-retrieve).
    ) -> Result<impl Iterator<Item = Result<S::Value, Box<dyn Error>>> + '_, StoreError> {
        let db_key = db_key.encode_key()?;
        let mut read_opts = ReadOptions::default();
        read_opts.set_iterate_range(rocksdb::PrefixRange::<&[u8]>(db_key.as_ref()));

        let mut db_iterator = self
            .db
            .raw_iterator_cf_opt(S::COLUMN_FAMILY, IteratorMode::Start, read_opts)
            .map_err(|e| StoreError::CFNotExist(e.to_string()))?;

        if skip_first {
            db_iterator.next();
        }

        Ok(db_iterator.take(limit).map(move |item| match item {
            Ok((key_bytes, _)) => {
                S::Value::decode_value(&key_bytes[db_key.len()..]).map_err(Into::into)
            }
            Err(err) => Err(err.into()),
        }))
    }
}

#[derive(Clone)]
pub struct CachedDbSetAccess<S: Schema, R = RandomState> {
    inner: DbSetAccess<S, R>,
    cache: DagCache<S::Key, Arc<RwLock<Vec<S::Value>>>>,
}
impl<S: Schema> CachedDbSetAccess<S> {
    pub fn new(db: Arc<DBStorage>, cache_size: usize) -> Self {
        Self {
            inner: DbSetAccess::<S>::new(db),
            cache: DagCache::new_with_capacity(cache_size),
        }
    }

    // Mark the key has been initialized in memory to speed up the read operation.
    pub fn initialize(&self, key: S::Key) {
        self.cache.insert(key, Arc::new(RwLock::new(Vec::new())));
    }

    pub fn read(&self, key: S::Key) -> Result<Arc<RwLock<Vec<S::Value>>>, StoreError> {
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
        S::Value: std::cmp::PartialEq,
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
