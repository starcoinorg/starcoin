// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cache_storage::GCacheStorage, upgrade::DBUpgrade};
use anyhow::{bail, Result};
use byteorder::{BigEndian, ReadBytesExt};
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_vm_types::state_store::table::TableHandle;
use std::{convert::TryInto, fmt::Debug, hash::Hash, marker::PhantomData, sync::Arc};
pub use {
    crate::batch::WriteBatch,
    starcoin_schemadb::{db::DBStorage, ColumnFamilyName, GWriteOp as WriteOp},
};

#[allow(clippy::upper_case_acronyms)]
pub trait KVStore: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn multiple_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>>;
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, key: Vec<u8>) -> Result<bool>;
    fn remove(&self, key: Vec<u8>) -> Result<()>;
    fn write_batch(&self, batch: WriteBatch) -> Result<()>;
    fn get_len(&self) -> Result<u64>;
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
    fn put_sync(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn write_batch_sync(&self, batch: WriteBatch) -> Result<()>;
}

pub trait InnerStore: Send + Sync {
    fn get_raw(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>>;
    fn put_raw(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool>;
    fn remove_raw(&self, prefix_name: &str, key: Vec<u8>) -> Result<()>;
    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()>;
    fn get_len(&self) -> Result<u64>;
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
    fn put_sync(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn write_batch_sync(&self, prefix_name: &str, batch: WriteBatch) -> Result<()>;
    fn multi_get(&self, prefix_name: &str, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>>;
}

pub type StorageInstance = GStorageInstance<Vec<u8>, Vec<u8>>;

///Generic Storage instance type define
#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum GStorageInstance<K, V>
where
    K: Hash + Eq + Default,
    V: Default,
{
    CACHE {
        cache: Arc<GCacheStorage<K, V>>,
    },
    DB {
        db: Arc<DBStorage>,
    },
    CacheAndDb {
        cache: Arc<GCacheStorage<K, V>>,
        db: Arc<DBStorage>,
    },
}

impl<K, V> GStorageInstance<K, V>
where
    K: Hash + Eq + Default,
    V: Default,
{
    pub fn new_cache_instance() -> Self {
        GStorageInstance::CACHE {
            cache: Arc::new(GCacheStorage::default()),
        }
    }
    pub fn new_db_instance(db: DBStorage) -> Self {
        Self::DB { db: Arc::new(db) }
    }

    pub fn new_cache_and_db_instance(cache: GCacheStorage<K, V>, db: DBStorage) -> Self {
        Self::CacheAndDb {
            cache: Arc::new(cache),
            db: Arc::new(db),
        }
    }

    pub fn cache(&self) -> Option<Arc<GCacheStorage<K, V>>> {
        match self {
            Self::CACHE { cache } | Self::CacheAndDb { cache, db: _ } => Some(cache.clone()),
            _ => None,
        }
    }

    pub fn db(&self) -> Option<&Arc<DBStorage>> {
        match self {
            Self::DB { db } | Self::CacheAndDb { cache: _, db } => Some(db),
            _ => None,
        }
    }

    // make sure Arc::strong_count(&db) == 1 unless will get None
    pub fn db_mut(&mut self) -> Option<&mut DBStorage> {
        match self {
            Self::DB { db } | Self::CacheAndDb { cache: _, db } => Arc::get_mut(db),
            _ => None,
        }
    }
}

impl StorageInstance {
    pub fn check_upgrade(&mut self) -> Result<()> {
        DBUpgrade::check_upgrade(self)
    }

    pub fn barnard_hard_fork(&mut self, config: Arc<NodeConfig>) -> Result<()> {
        if config.net().id().chain_id().is_barnard() {
            info!("barnard_hard_fork in");
            return DBUpgrade::barnard_hard_fork(self);
        }
        Ok(())
    }
}

impl InnerStore for StorageInstance {
    fn get_raw(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        match self {
            StorageInstance::CACHE { cache } => cache.get_raw(prefix_name, key),
            StorageInstance::DB { db } => db.get_no_schema(prefix_name, &key),
            StorageInstance::CacheAndDb { cache, db } => {
                // first get from cache
                // if from cache get non-existent, query from db
                if let Ok(Some(value)) = cache.get_raw(prefix_name, key.clone()) {
                    Ok(Some(value))
                } else {
                    match db.get_no_schema(prefix_name, &key)? {
                        Some(value) => {
                            // cache.put_obj(prefix_name, key, CacheObject::Value(value.clone()))?;
                            Ok(Some(value))
                        }
                        None => {
                            // put null vec to cache for avoid repeatedly querying non-existent data from db
                            // cache.put_obj(prefix_name, key, CACHE_NONE_OBJECT.clone())?;
                            Ok(None)
                        }
                    }
                }
            }
        }
    }

    fn put_raw(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        match self {
            StorageInstance::CACHE { cache } => cache.put_raw(prefix_name, key, value),
            StorageInstance::DB { db } => db.put_no_schema(prefix_name, &key, &value),
            StorageInstance::CacheAndDb { cache, db } => db
                .put_no_schema(prefix_name, &key, &value)
                .and_then(|_| cache.put_raw(prefix_name, key, value)),
        }
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        match self {
            StorageInstance::CACHE { cache } => cache.contains_key(prefix_name, key),
            StorageInstance::DB { db } => db.contains_key(prefix_name, &key),
            StorageInstance::CacheAndDb { cache, db } => {
                match cache.contains_key(prefix_name, key.clone()) {
                    Ok(true) => Ok(true),
                    _ => db.contains_key(prefix_name, &key),
                }
            }
        }
    }

    fn remove_raw(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
        match self {
            StorageInstance::CACHE { cache } => cache.remove_raw(prefix_name, key),
            StorageInstance::DB { db } => db.remove_no_schema(prefix_name, &key),
            StorageInstance::CacheAndDb { cache, db } => {
                match db.remove_no_schema(prefix_name, &key) {
                    Ok(_) => cache.remove_raw(prefix_name, key),
                    _ => bail!("db storage remove error."),
                }
            }
        }
    }

    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        match self {
            StorageInstance::CACHE { cache } => cache.write_batch(prefix_name, batch),
            StorageInstance::DB { db } => {
                db.write_batch_inner(prefix_name, batch.rows.as_ref(), true)
            }
            StorageInstance::CacheAndDb { cache, db } => {
                match db.write_batch_inner(prefix_name, batch.rows.as_ref(), true) {
                    Ok(_) => cache.write_batch(prefix_name, batch),
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

    fn put_sync(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let mut opts = rocksdb::WriteOptions::default();
        opts.set_sync(true);
        match self {
            StorageInstance::CACHE { cache } => cache.put_raw(prefix_name, key, value),
            StorageInstance::DB { db } => db.put_no_schema_opt(prefix_name, &key, &value, &opts),
            StorageInstance::CacheAndDb { cache, db } => db
                .put_no_schema_opt(prefix_name, &key, &value, &opts)
                .and_then(|_| cache.put_raw(prefix_name, key, value)),
        }
    }

    fn write_batch_sync(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        match self {
            StorageInstance::CACHE { cache } => cache.write_batch(prefix_name, batch),
            StorageInstance::DB { db } => db.write_batch_inner(prefix_name, &batch.rows, true),
            StorageInstance::CacheAndDb { cache, db } => {
                match db.write_batch_inner(prefix_name, &batch.rows, true) {
                    Ok(_) => cache.write_batch(prefix_name, batch),
                    Err(err) => bail!("write batch db error: {}", err),
                }
            }
        }
    }

    fn multi_get(&self, prefix_name: &str, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>> {
        match self {
            StorageInstance::CACHE { cache } => cache.multi_get(prefix_name, keys),
            StorageInstance::DB { db } => db.multi_get_no_schema(prefix_name, &keys),
            StorageInstance::CacheAndDb { db, .. } => {
                /* https://github.com/facebook/rocksdb/wiki/Block-Cache#lru-cache
                * if use multi_get from CacheStorage, cache may evict some records
                    should check every Option<Vec<u8>> is not none, if have none
                    we need multi_get these from db
                    test db multi_get performance is better than CacheStorage
                 */

                /*
                let mut result = cache.multi_get(prefix_name, keys.clone())?;
                let mut db_idxs = vec![];
                let mut db_keys = vec![];
                // cache may be evict record
                for (idx, item) in result.iter().enumerate() {
                    if item.is_none() {
                        db_idxs.push(idx);
                        if let Some(key) = keys.get(idx) {
                            db_keys.push(key.clone());
                        }
                    }
                }
                if db_idxs.is_empty() {
                    return Ok(result);
                }
                let values = db.multi_get(prefix_name, db_keys)?;
                for (idx, value) in db_idxs.into_iter().zip(values) {
                    if let Some(res) = result.get_mut(idx) {
                        *res = value;
                    }
                }
                Ok(result)
                 */
                db.multi_get_no_schema(prefix_name, &keys)
            }
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

    pub fn storage(&self) -> &StorageInstance {
        &self.instance
    }
}

impl<CF> KVStore for InnerStorage<CF>
where
    CF: ColumnFamily,
{
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.instance.get_raw(self.prefix_name, key.to_vec())
    }

    fn multiple_get(&self, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>> {
        self.instance.multi_get(self.prefix_name, keys)
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.instance.put_raw(self.prefix_name, key, value)
    }

    fn contains_key(&self, key: Vec<u8>) -> Result<bool> {
        self.instance.contains_key(self.prefix_name, key)
    }

    fn remove(&self, key: Vec<u8>) -> Result<()> {
        self.instance.remove_raw(self.prefix_name, key)
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

    fn put_sync(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.instance.put_sync(self.prefix_name, key, value)
    }

    fn write_batch_sync(&self, batch: WriteBatch) -> Result<()> {
        self.instance.write_batch_sync(self.prefix_name, batch)
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
pub struct CodecWriteBatch<K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    rows: Vec<WriteOp<K, V>>,
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
        rows.extend(kvs.into_iter().map(|(k, v)| (WriteOp::Value(k, v))));
        Self { rows }
    }

    pub fn new_deletes(ks: Vec<K>) -> Self {
        let mut rows = Vec::new();
        rows.extend(ks.into_iter().map(|k| WriteOp::Deletion(k)));
        Self { rows }
    }

    /// Adds an insert/update operation to the batch.
    pub fn put(&mut self, key: K, value: V) -> Result<()> {
        self.rows.push(WriteOp::Value(key, value));
        Ok(())
    }

    /// Adds a delete operation to the batch.
    pub fn delete(&mut self, key: K) -> Result<()> {
        self.rows.push(WriteOp::Deletion(key));
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
    type Item = WriteOp<K, V>;
    type IntoIter = std::vec::IntoIter<WriteOp<K, V>>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

#[allow(clippy::upper_case_acronyms)]
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

    fn put_raw(&self, key: K, value: Vec<u8>) -> Result<()>;

    fn get_raw(&self, key: K) -> Result<Option<Vec<u8>>>;

    //    fn iter(&self) -> Result<SchemaIterator<K, V>>;
}

impl KeyCodec for u64 {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    #[allow(clippy::redundant_slicing)]
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
        bcs_ext::to_bytes(self)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(data)
    }
}

impl KeyCodec for Vec<u8> {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok(data.to_vec())
    }
}

impl ValueCodec for Vec<u8> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(data.to_vec())
    }
}

impl KeyCodec for TableHandle {
    fn encode_key(&self) -> Result<Vec<u8>> {
        bcs_ext::to_bytes(self)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(data)
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

    fn put_raw(&self, key: K, value: Vec<u8>) -> Result<()> {
        KVStore::put(self.get_store(), key.encode_key()?, value)
    }

    fn get_raw(&self, key: K) -> Result<Option<Vec<u8>>> {
        KVStore::get(self.get_store(), key.encode_key()?.as_slice())
    }

    //fn iter(&self) -> Result<SchemaIterator<K, V>> {
    //    let db = self
    //        .get_store()
    //        .storage()
    //        .db()
    //        .ok_or_else(|| format_err!("Only support scan on db storage instance"))?;
    //    db.iter_raw::<K, V>(self.get_store().prefix_name)
    //    unimplemented!()
    //}
}
