// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use anyhow::{bail, Result};
use crypto::HashValue;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
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
    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()>;
    fn get_len(&self) -> Result<u64>;
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
}

pub static CACHE_NONE_OBJECT: Lazy<CacheObject> = Lazy::new(|| CacheObject::None);
/// Define cache object distinguish between normal objects and missing
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CacheObject {
    Value(Vec<u8>),
    None,
}

impl From<&CacheObject> for Option<Vec<u8>> {
    fn from(cache_obj: &CacheObject) -> Option<Vec<u8>> {
        match cache_obj.clone() {
            CacheObject::Value(v) => Some(v),
            CacheObject::None => None,
        }
    }
}

///Storage instance type define
#[derive(Clone)]
pub enum StorageInstance {
    CACHE {
        cache: Arc<CacheStorage>,
    },
    DB {
        db: Arc<dyn InnerStore>,
    },
    CacheAndDb {
        cache: Arc<CacheStorage>,
        db: Arc<dyn InnerStore>,
    },
}

impl StorageInstance {
    pub fn new_cache_instance() -> Self {
        StorageInstance::CACHE {
            cache: Arc::new(CacheStorage::new()),
        }
    }
    pub fn new_db_instance(db: Arc<DBStorage>) -> Self {
        Self::DB { db }
    }
    pub fn new_cache_and_db_instance(cache: Arc<CacheStorage>, db: Arc<DBStorage>) -> Self {
        Self::CacheAndDb { cache, db }
    }
}

impl InnerStore for StorageInstance {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        match self {
            StorageInstance::CACHE { cache } => cache.get(prefix_name, key),
            StorageInstance::DB { db } => db.get(prefix_name, key),
            StorageInstance::CacheAndDb { cache, db } => {
                // first get from cache
                if let Ok(Some(cache_obj)) = cache.get_obj(prefix_name, key.clone()) {
                    match cache_obj {
                        CacheObject::Value(value) => Ok(Some(value)),
                        CacheObject::None => Ok(None),
                    }
                } else {
                    match db.get(prefix_name, key.clone())? {
                        Some(value) => {
                            cache.put_obj(prefix_name, key, CacheObject::Value(value.clone()))?;
                            Ok(Some(value))
                        }
                        None => {
                            // put null vec to cache for avoid repeatedly querying non-existent data from db
                            cache.put_obj(prefix_name, key, CACHE_NONE_OBJECT.clone())?;
                            Ok(None)
                        }
                    }
                }
            }
        }
    }

    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        match self {
            StorageInstance::CACHE { cache } => cache.put(prefix_name, key, value),
            StorageInstance::DB { db } => db.put(prefix_name, key, value),
            StorageInstance::CacheAndDb { cache, db } => db
                .put(prefix_name, key.clone(), value.clone())
                .and_then(|_| cache.put_obj(prefix_name, key, CacheObject::Value(value))),
        }
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        match self {
            StorageInstance::CACHE { cache } => cache.contains_key(prefix_name, key),
            StorageInstance::DB { db } => db.contains_key(prefix_name, key),
            StorageInstance::CacheAndDb { cache, db } => {
                match cache.get_obj(prefix_name, key.clone()) {
                    Ok(Some(cache_obj)) => match cache_obj {
                        CacheObject::Value(_value) => Ok(true),
                        CacheObject::None => Ok(false),
                    },
                    _ => db.contains_key(prefix_name, key),
                }
            }
        }
    }

    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
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

    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        match self {
            StorageInstance::CACHE { cache } => cache.write_batch(prefix_name, batch),
            StorageInstance::DB { db } => db.write_batch(prefix_name, batch),
            StorageInstance::CacheAndDb { cache, db } => {
                match db.write_batch(prefix_name, batch.clone()) {
                    Ok(_) => cache.write_batch_obj(prefix_name, batch),
                    Err(err) => bail!("write batch db error: {}", err),
                }
            }
        }
    }
    fn get_len(&self) -> Result<u64> {
        match self {
            StorageInstance::CACHE { cache } => cache.get_len(),
            StorageInstance::CacheAndDb { cache, db: _ } => cache.get_len(),
            _ => bail!("DB instance not support get length method!"),
        }
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>> {
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
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.instance.get(self.prefix_name, key.to_vec())
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.instance.put(self.prefix_name, key, value)
    }

    fn contains_key(&self, key: Vec<u8>) -> Result<bool> {
        self.instance.contains_key(self.prefix_name, key)
    }

    fn remove(&self, key: Vec<u8>) -> Result<()> {
        self.instance.remove(self.prefix_name, key)
    }

    fn write_batch(&self, batch: WriteBatch) -> Result<()> {
        self.instance.write_batch(self.prefix_name, batch)
    }

    fn get_len(&self) -> Result<u64> {
        self.instance.get_len()
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>> {
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

#[derive(Clone)]
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

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok(HashValue::from_slice(data)?)
    }
}

impl ValueCodec for HashValue {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(HashValue::from_slice(data)?)
    }
}
