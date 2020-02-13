// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use crypto::{hash::CryptoHash, HashValue};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

pub trait KVStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
}

pub struct HashMapKVStore {
    map: RefCell<HashMap<Vec<u8>, Vec<u8>>>,
}

impl HashMapKVStore {
    pub fn new() -> Self {
        HashMapKVStore {
            map: RefCell::new(HashMap::new()),
        }
    }
}

impl KVStore for HashMapKVStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.map.borrow().get(key).map(|v| v.to_vec()))
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.map.borrow_mut().insert(key, value);
        Ok(())
    }
}

pub trait KeyCodec: Sized + PartialEq + Debug {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_key(&self) -> Result<Vec<u8>>;
}

pub trait ValueCodec: Sized + PartialEq + Debug {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_value(&self) -> Result<Vec<u8>>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_value(data: &[u8]) -> Result<Self>;
}

pub struct CodecKVStore<K, V>
where
    K: KeyCodec,
    V: ValueCodec,
{
    store: Arc<dyn KVStore>,
    k: PhantomData<K>,
    v: PhantomData<V>,
}

impl<K, V> CodecKVStore<K, V>
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
}

impl KeyCodec for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }
}
