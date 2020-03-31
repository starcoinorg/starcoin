// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
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

pub trait Repository: Send + Sync {
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

/// Define simple storage package for one storage
pub struct InnerStorage {
    repository: Arc<dyn InnerStore>,
    pub prefix_name: ColumnFamilyName,
}
impl InnerStorage {
    pub fn new(repository: Arc<dyn InnerStore>, prefix_name: ColumnFamilyName) -> Self {
        Self {
            repository,
            prefix_name,
        }
    }
}

impl Repository for InnerStorage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        self.repository.clone().get(self.prefix_name, key.to_vec())
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        self.repository.clone().put(self.prefix_name, key, value)
    }

    fn contains_key(&self, key: Vec<u8>) -> Result<bool, Error> {
        self.repository.clone().contains_key(self.prefix_name, key)
    }

    fn remove(&self, key: Vec<u8>) -> Result<(), Error> {
        self.repository.clone().remove(self.prefix_name, key)
    }

    fn write_batch(&self, batch: WriteBatch) -> Result<(), Error> {
        self.repository.write_batch(batch)
    }

    fn get_len(&self) -> Result<u64, Error> {
        self.repository.clone().get_len()
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>, Error> {
        self.repository.clone().keys()
    }
}

/// two level storage package
pub struct Storage {
    cache: Arc<dyn InnerStore>,
    db: Arc<dyn InnerStore>,
    pub prefix_name: ColumnFamilyName,
}

impl Storage {
    pub fn new(
        cache: Arc<dyn InnerStore>,
        db: Arc<dyn InnerStore>,
        prefix_name: ColumnFamilyName,
    ) -> Self {
        Storage {
            cache,
            db,
            prefix_name,
        }
    }
}

impl Repository for Storage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        // first get from cache
        let key_vec = key.to_vec();
        if let Ok(Some(v)) = self.cache.clone().get(self.prefix_name, key_vec.clone()) {
            Ok(Some(v))
        } else {
            self.db.clone().get(self.prefix_name, key_vec.clone())
        }
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        self.db
            .clone()
            .put(self.prefix_name, key.clone(), value.clone())
            .unwrap();
        self.cache.clone().put(self.prefix_name, key, value)
    }

    fn contains_key(&self, key: Vec<u8>) -> Result<bool, Error> {
        self.cache.clone().contains_key(self.prefix_name, key)
    }

    fn remove(&self, key: Vec<u8>) -> Result<(), Error> {
        match self.db.clone().remove(self.prefix_name, key.clone()) {
            Ok(_) => self.cache.clone().remove(self.prefix_name, key),
            Err(err) => bail!("remove persistence error: {}", err),
        }
    }

    fn write_batch(&self, batch: WriteBatch) -> Result<(), Error> {
        match self.db.write_batch(batch.clone()) {
            Ok(_) => self.cache.write_batch(batch),
            Err(err) => bail!("write batch db error: {}", err),
        }
    }

    fn get_len(&self) -> Result<u64, Error> {
        self.cache.get_len()
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>, Error> {
        self.cache.keys()
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
    store: Arc<dyn Repository>,
    k: PhantomData<K>,
    v: PhantomData<V>,
}

impl<K, V> CodecStorage<K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    pub fn new(store: Arc<dyn Repository>) -> Self {
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
