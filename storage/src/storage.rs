// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use anyhow::{bail, Error, Result};
use crypto::HashValue;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

/// Type alias to improve readability.
pub type ColumnFamilyName = &'static str;

#[derive(Debug, Clone)]
pub enum WriteOp {
    Value(Vec<u8>),
    Deletion,
}

pub trait KVStore: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, key: Vec<u8>) -> Result<bool>;
    fn remove(&self, key: Vec<u8>) -> Result<()>;
    fn write_batch(&self, batch: WriteBatch) -> Result<()>;
    fn get_len(&self) -> Result<u64>;
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
}

pub trait InnerStore: Send + Sync {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>>;
    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool>;
    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()>;
    fn write_batch(&self, batch: WriteBatch) -> Result<()>;
    fn get_len(&self) -> Result<u64>;
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
}

///Storage instance type define
//#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Clone)]
pub enum StorageInstance {
    CACHE {
        cache: Arc<dyn InnerStore>,
    },
    DB {
        db: Arc<dyn InnerStore>,
    },
    CacheAndDb {
        cache: Arc<dyn InnerStore>,
        db: Arc<dyn InnerStore>,
    },
}
impl StorageInstance {
    pub fn new_cache_instance(cache: CacheStorage) -> Self {
        StorageInstance::CACHE {
            cache: Arc::new(cache),
        }
    }
    pub fn new_db_instance(db: DBStorage) -> Self {
        Self::DB { db: Arc::new(db) }
    }
    pub fn new_cache_and_db_instance(cache: Arc<CacheStorage>, db: Arc<DBStorage>) -> Self {
        Self::CacheAndDb { cache, db }
    }
}
impl InnerStore for StorageInstance {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        match self {
            StorageInstance::CACHE { cache } => cache.get(prefix_name, key.clone()),
            StorageInstance::DB { db } => db.get(prefix_name, key.clone()),
            StorageInstance::CacheAndDb { cache, db } => {
                // first get from cache
                if let Ok(Some(v)) = cache.get(prefix_name, key.clone()) {
                    Ok(Some(v))
                } else {
                    db.get(prefix_name, key.clone())
                }
            }
        }
    }

    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        match self {
            StorageInstance::CACHE { cache } => cache.put(prefix_name, key, value),
            StorageInstance::DB { db } => db.put(prefix_name, key, value),
            StorageInstance::CacheAndDb { cache, db } => {
                db.put(prefix_name, key.clone(), value.clone()).unwrap();
                cache.put(prefix_name, key, value)
            }
        }
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool, Error> {
        match self {
            StorageInstance::CACHE { cache } => cache.contains_key(prefix_name, key),
            StorageInstance::DB { db } => db.contains_key(prefix_name, key),
            StorageInstance::CacheAndDb { cache, db } => {
                match cache.contains_key(prefix_name, key.clone()) {
                    Err(_) => db.contains_key(prefix_name, key.clone()),
                    Ok(is_contains) => Ok(is_contains),
                }
            }
        }
    }

    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<(), Error> {
        match self {
            StorageInstance::CACHE { cache } => cache.remove(prefix_name, key),
            StorageInstance::DB { db } => db.remove(prefix_name, key),
            StorageInstance::CacheAndDb { cache, db } => {
                match db.remove(prefix_name, key.clone()) {
                    Ok(_) => cache.remove(prefix_name, key),
                    _ => bail!("db storage remove error."),
                }
            }
        }
    }

    fn write_batch(&self, batch: WriteBatch) -> Result<(), Error> {
        match self {
            StorageInstance::CACHE { cache } => cache.write_batch(batch),
            StorageInstance::DB { db } => db.write_batch(batch),
            StorageInstance::CacheAndDb { cache, db } => match db.write_batch(batch.clone()) {
                Ok(_) => cache.write_batch(batch),
                Err(err) => bail!("write batch db error: {}", err),
            },
        }
    }
    fn get_len(&self) -> Result<u64, Error> {
        match self {
            StorageInstance::CACHE { cache } => cache.get_len(),
            StorageInstance::CacheAndDb { cache, db: _ } => cache.get_len(),
            _ => bail!("DB instance not support get length method!"),
        }
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>, Error> {
        match self {
            StorageInstance::CACHE { cache } => cache.keys(),
            StorageInstance::CacheAndDb { cache, db: _ } => cache.keys(),
            _ => bail!("DB instance not support keys method!"),
        }
    }
}

/// Define inner storage implement
pub struct InnerStorage {
    pub prefix_name: ColumnFamilyName,
    instance: StorageInstance,
}

impl InnerStorage {
    pub fn new(instance: StorageInstance, prefix_name: ColumnFamilyName) -> Self {
        Self {
            instance,
            prefix_name,
        }
    }
}

impl KVStore for InnerStorage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        self.instance.get(self.prefix_name, key.to_vec())
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        self.instance.put(self.prefix_name, key, value)
    }

    fn contains_key(&self, key: Vec<u8>) -> Result<bool, Error> {
        self.instance.contains_key(self.prefix_name, key)
    }

    fn remove(&self, key: Vec<u8>) -> Result<(), Error> {
        self.instance.remove(self.prefix_name, key)
    }

    fn write_batch(&self, batch: WriteBatch) -> Result<(), Error> {
        self.instance.write_batch(batch)
    }

    fn get_len(&self) -> Result<u64, Error> {
        self.instance.get_len()
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>, Error> {
        self.instance.keys()
    }
}

pub trait KeyCodec: Sized + PartialEq + Debug {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_key(&self) -> Result<Vec<u8>>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_key(data: &[u8]) -> Result<Self>;
}

pub trait ValueCodec: Sized + PartialEq + Debug {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_value(&self) -> Result<Vec<u8>>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_value(data: &[u8]) -> Result<Self>;
}

pub struct CodecStorage<K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    store: Arc<dyn KVStore>,
    k: PhantomData<K>,
    v: PhantomData<V>,
}

impl<K, V> CodecStorage<K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    pub fn new(store: Arc<dyn KVStore>) -> Self {
        Self {
            store,
            k: PhantomData,
            v: PhantomData,
        }
    }

    pub fn get(&self, key: K) -> Result<Option<V>> {
        match self.store.get(key.encode_key()?.as_slice())? {
            Some(v) => Ok(Some(V::decode_value(v.as_slice())?)),
            None => Ok(None),
        }
    }
    pub fn put(&self, key: K, value: V) -> Result<()> {
        self.store.put(key.encode_key()?, value.encode_value()?)
    }
    pub fn contains_key(&self, key: K) -> Result<bool> {
        self.store.contains_key(key.encode_key()?)
    }
    pub fn remove(&self, key: K) -> Result<()> {
        self.store.remove(key.encode_key()?)
    }

    pub fn write_batch(&self, batch: WriteBatch) -> Result<()> {
        self.store.write_batch(batch)
    }

    pub fn get_len(&self) -> Result<u64> {
        self.store.get_len()
    }
    pub fn keys(&self) -> Result<Vec<Vec<u8>>> {
        self.store.keys()
    }
}

impl KeyCodec for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, Error> {
        Ok(HashValue::from_slice(data)?)
    }
}

impl ValueCodec for HashValue {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self, Error> {
        Ok(HashValue::from_slice(data)?)
    }
}
