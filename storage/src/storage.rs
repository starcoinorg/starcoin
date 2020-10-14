// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use crate::batch::WriteBatch;
use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use anyhow::{bail, Result};
use byteorder::{BigEndian, ReadBytesExt};
use crypto::HashValue;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

/// Type alias to improve readability.
pub type ColumnFamilyName = &'static str;

pub trait KVStore: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn multiple_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>> {
        //TODO optimize
        keys.into_iter().map(|k| self.get(k.as_slice())).collect()
    }
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
        db: Arc<DBStorage>,
    },
    CacheAndDb {
        cache: Arc<CacheStorage>,
        db: Arc<DBStorage>,
    },
}

impl StorageInstance {
    pub fn new_cache_instance() -> Self {
        StorageInstance::CACHE {
            cache: Arc::new(CacheStorage::new()),
        }
    }
    pub fn new_db_instance(db: DBStorage) -> Self {
        Self::DB { db: Arc::new(db) }
    }

    pub fn new_cache_and_db_instance(cache: CacheStorage, db: DBStorage) -> Self {
        Self::CacheAndDb {
            cache: Arc::new(cache),
            db: Arc::new(db),
        }
    }

    pub fn cache(&self) -> Option<Arc<CacheStorage>> {
        match self {
            StorageInstance::CACHE { cache } | StorageInstance::CacheAndDb { cache, db: _ } => {
                Some(cache.clone())
            }
            _ => None,
        }
    }

    pub fn db(&self) -> Option<Arc<DBStorage>> {
        match self {
            StorageInstance::DB { db } | StorageInstance::CacheAndDb { cache: _, db } => {
                Some(db.clone())
            }
            _ => None,
        }
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

pub trait ColumnFamily: Send + Sync {
    type Key;
    type Value;
    fn name() -> ColumnFamilyName;
}

/// Define inner storage implement
#[derive(Clone)]
pub struct InnerStorage<CF>
where
    CF: ColumnFamily,
{
    pub prefix_name: ColumnFamilyName,
    instance: StorageInstance,
    cf: PhantomData<CF>,
}

impl<CF> InnerStorage<CF>
where
    CF: ColumnFamily,
{
    pub fn new(instance: StorageInstance) -> Self {
        Self {
            instance,
            prefix_name: CF::name(),
            cf: PhantomData,
        }
    }
}

impl<CF> KVStore for InnerStorage<CF>
where
    CF: ColumnFamily,
{
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

pub trait SchemaStorage: Sized + ColumnFamily {
    fn get_store(&self) -> &InnerStorage<Self>;
}

pub trait KeyCodec: Clone + Sized + Debug + std::marker::Send + std::marker::Sync {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_key(&self) -> Result<Vec<u8>>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_key(data: &[u8]) -> Result<Self>;
}

pub trait ValueCodec: Clone + Sized + Debug + std::marker::Send + std::marker::Sync {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_value(&self) -> Result<Vec<u8>>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_value(data: &[u8]) -> Result<Self>;
}

#[derive(Debug, Clone)]
pub enum WriteOp<V> {
    Value(V),
    Deletion,
}

impl<V> WriteOp<V>
where
    V: ValueCodec,
{
    pub fn into_raw_op(self) -> Result<WriteOp<Vec<u8>>> {
        Ok(match self {
            WriteOp::Value(v) => WriteOp::Value(v.encode_value()?),
            WriteOp::Deletion => WriteOp::Deletion,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CodecWriteBatch<K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    rows: Vec<(K, WriteOp<V>)>,
}

impl<K, V> Default for CodecWriteBatch<K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    fn default() -> Self {
        Self { rows: Vec::new() }
    }
}

impl<K, V> CodecWriteBatch<K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    /// Creates an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_puts(kvs: Vec<(K, V)>) -> Self {
        let mut rows = Vec::new();
        rows.extend(kvs.into_iter().map(|(k, v)| (k, WriteOp::Value(v))));
        Self { rows }
    }

    pub fn new_deletes(ks: Vec<K>) -> Self {
        let mut rows = Vec::new();
        rows.extend(ks.into_iter().map(|k| (k, WriteOp::Deletion)));
        Self { rows }
    }

    /// Adds an insert/update operation to the batch.
    pub fn put(&mut self, key: K, value: V) -> Result<()> {
        self.rows.push((key, WriteOp::Value(value)));
        Ok(())
    }

    /// Adds a delete operation to the batch.
    pub fn delete(&mut self, key: K) -> Result<()> {
        self.rows.push((key, WriteOp::Deletion));
        Ok(())
    }

    ///Clear all operation to the next batch.
    pub fn clear(&mut self) -> Result<()> {
        self.rows.clear();
        Ok(())
    }
}

impl<K, V> IntoIterator for CodecWriteBatch<K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    type Item = (K, WriteOp<V>);
    type IntoIter = std::vec::IntoIter<(K, WriteOp<V>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

pub trait CodecKVStore<K, V>: std::marker::Send + std::marker::Sync
where
    K: KeyCodec,
    V: ValueCodec,
{
    fn get(&self, key: K) -> Result<Option<V>>;

    fn multiple_get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>>;

    fn put(&self, key: K, value: V) -> Result<()>;

    fn contains_key(&self, key: K) -> Result<bool>;

    fn remove(&self, key: K) -> Result<()>;

    fn write_batch(&self, batch: CodecWriteBatch<K, V>) -> Result<()>;

    fn put_all(&self, kvs: Vec<(K, V)>) -> Result<()> {
        self.write_batch(CodecWriteBatch::new_puts(kvs))
    }

    fn delete_all(&self, ks: Vec<K>) -> Result<()> {
        self.write_batch(CodecWriteBatch::new_deletes(ks))
    }

    fn get_len(&self) -> Result<u64>;

    fn keys(&self) -> Result<Vec<K>>;
}

impl KeyCodec for u64 {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok((&data[..]).read_u64::<BigEndian>()?)
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

impl ValueCodec for Vec<HashValue> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        scs::to_bytes(self)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        scs::from_bytes(data)
    }
}

impl<K, V, S> CodecKVStore<K, V> for S
where
    K: KeyCodec,
    V: ValueCodec,
    S: SchemaStorage,
    S: ColumnFamily<Key = K, Value = V>,
{
    fn get(&self, key: K) -> Result<Option<V>> {
        match KVStore::get(self.get_store(), key.encode_key()?.as_slice())? {
            Some(value) => Ok(Some(<V>::decode_value(value.as_slice())?)),
            None => Ok(None),
        }
    }

    fn multiple_get(&self, keys: Vec<K>) -> Result<Vec<Option<V>>> {
        let encoded_keys: Result<Vec<Vec<u8>>> =
            keys.into_iter().map(|key| key.encode_key()).collect();
        let values = KVStore::multiple_get(self.get_store(), encoded_keys?)?;
        values
            .into_iter()
            .map(|value| match value {
                Some(value) => Ok(Some(<V>::decode_value(value.as_slice())?)),
                None => Ok(None),
            })
            .collect()
    }

    fn put(&self, key: K, value: V) -> Result<()> {
        KVStore::put(self.get_store(), key.encode_key()?, value.encode_value()?)
    }

    fn contains_key(&self, key: K) -> Result<bool> {
        KVStore::contains_key(self.get_store(), key.encode_key()?)
    }

    fn remove(&self, key: K) -> Result<()> {
        KVStore::remove(self.get_store(), key.encode_key()?)
    }

    fn write_batch(&self, batch: CodecWriteBatch<K, V>) -> Result<()> {
        KVStore::write_batch(self.get_store(), batch.try_into()?)
    }

    fn get_len(&self) -> Result<u64> {
        KVStore::get_len(self.get_store())
    }

    fn keys(&self) -> Result<Vec<K>> {
        let keys = KVStore::keys(self.get_store())?;
        keys.into_iter()
            .map(|key| <K>::decode_key(key.as_slice()))
            .collect()
    }
}
