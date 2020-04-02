// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::storage::{InnerStore, WriteOp};
use anyhow::{Error, Result};
use lru::LruCache;
use parking_lot::Mutex;

const LRU_CACHE_DEFAULT_SIZE: usize = 65535;

pub struct CacheStorage {
    cache: Mutex<LruCache<Vec<u8>, Vec<u8>>>,
}

impl CacheStorage {
    pub fn new() -> Self {
        CacheStorage {
            cache: Mutex::new(LruCache::new(LRU_CACHE_DEFAULT_SIZE)),
        }
    }
    pub fn new_with_capacity(size: usize) -> Self {
        CacheStorage {
            cache: Mutex::new(LruCache::new(size)),
        }
    }
}

impl InnerStore for CacheStorage {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        let compose = compose_key(prefix_name.to_string(), key)?;
        Ok(self.cache.lock().get(&compose).map(|v| v.to_vec()))
    }

    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        &self
            .cache
            .lock()
            .put(compose_key(prefix_name.to_string(), key)?, value);
        Ok(())
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        let compose = compose_key(prefix_name.to_string(), key)?;
        Ok(self.cache.lock().contains(&compose))
    }
    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
        let compose = compose_key(prefix_name.to_string(), key)?;
        self.cache.lock().pop(&compose).unwrap();
        Ok(())
    }

    fn write_batch(&self, batch: WriteBatch) -> Result<(), Error> {
        for (prefix_name, rows) in &batch.rows {
            for (key, write_op) in rows {
                match write_op {
                    WriteOp::Value(value) => {
                        self.put(prefix_name, key.to_vec(), value.to_vec()).unwrap()
                    }
                    WriteOp::Deletion => self.remove(prefix_name, key.to_vec()).unwrap(),
                };
            }
        }
        Ok(())
    }

    fn get_len(&self) -> Result<u64, Error> {
        Ok(self.cache.lock().len() as u64)
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>, Error> {
        let mut all_keys = vec![];
        for (key, _) in self.cache.lock().iter() {
            all_keys.push(key.to_vec());
        }
        Ok(all_keys)
    }
}

fn compose_key(prefix_name: String, source_key: Vec<u8>) -> Result<Vec<u8>> {
    let temp_vec = prefix_name.as_bytes().to_vec();
    let mut compose = Vec::with_capacity(temp_vec.len() + source_key.len());
    compose.extend(temp_vec);
    compose.extend(source_key);
    Ok(compose)
}
