// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::memory_storage::MemoryCache;
use crate::persistence_storage::PersistenceStorage;
use crate::storage::Store::{MemoryStore, PersistenceStore};
use anyhow::{bail, Error, Result};

pub trait Repository {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, key: Vec<u8>) -> Result<bool>;
    fn remove(&self, key: Vec<u8>) -> Result<()>;
}

pub struct Storage {
    cache: Store,
    persistence: Store,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            cache: MemoryStore(MemoryCache::new()),
            persistence: PersistenceStore(PersistenceStorage::new()),
        }
    }
}

impl Repository for Storage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        // first get from cache
        match self.cache.get(key) {
            Ok(v) => Ok(v),
            _ => self.persistence.get(key),
        }
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        unimplemented!()
    }

    fn contains_key(&self, key: Vec<u8>) -> Result<bool, Error> {
        unimplemented!()
    }

    fn remove(&self, key: Vec<u8>) -> Result<(), Error> {
        unimplemented!()
    }
}

pub enum Store {
    MemoryStore(MemoryCache),
    PersistenceStore(PersistenceStorage),
}

impl Store {
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        match *self {
            MemoryStore(ref c) => c.get(key),
            PersistenceStore(ref c) => c.get(key),
        }
    }

    pub fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        unimplemented!()
    }
    pub fn contains_key(&self, key: Vec<u8>) -> Result<bool> {
        unimplemented!()
    }
    pub fn remove(&self, key: Vec<u8>) -> Result<()> {
        unimplemented!()
    }
}
