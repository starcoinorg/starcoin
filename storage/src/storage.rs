// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::memory_storage::MemoryCache;
use crate::persistence_storage::PersistenceStorage;
use anyhow::{bail, Error, Result};
use std::sync::Arc;

pub trait Repository {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn contains_key(&self, key: Vec<u8>) -> Result<bool>;
    fn remove(&self, key: Vec<u8>) -> Result<()>;
}

pub struct Storage {
    cache: Arc<dyn Repository>,
    persistence: Arc<dyn Repository>,
}

impl Storage {
    pub fn new(cache: Arc<dyn Repository>, persistence: Arc<dyn Repository>) -> Self {
        Storage { cache, persistence }
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
