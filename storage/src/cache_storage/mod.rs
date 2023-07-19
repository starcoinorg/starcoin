// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    batch::WriteBatch,
    metrics::{record_metrics, StorageMetrics},
    storage::{InnerStore, WriteOp},
};
use anyhow::{Error, Result};
use lru::LruCache;
use parking_lot::Mutex;
use starcoin_config::DEFAULT_CACHE_SIZE;

pub struct CacheStorage {
    cache: Mutex<LruCache<Vec<u8>, Vec<u8>>>,
    metrics: Option<StorageMetrics>,
}

impl CacheStorage {
    pub fn new(metrics: Option<StorageMetrics>) -> Self {
        CacheStorage {
            cache: Mutex::new(LruCache::new(DEFAULT_CACHE_SIZE)),
            metrics,
        }
    }
    pub fn new_with_capacity(size: usize, metrics: Option<StorageMetrics>) -> Self {
        CacheStorage {
            cache: Mutex::new(LruCache::new(size)),
            metrics,
        }
    }
    pub fn remove_all(&self) {
        self.cache.lock().clear();
    }
}

impl Default for CacheStorage {
    fn default() -> Self {
        Self::new(None)
    }
}

impl InnerStore for CacheStorage {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        record_metrics("cache", prefix_name, "get", self.metrics.as_ref())
            .call(|| Ok(self.get_inner(Some(prefix_name), key)))
    }

    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        // remove record_metrics for performance
        // record_metrics add in write_batch to reduce Instant::now system call
        let len = self.put_inner(Some(prefix_name), key, value);
        if let Some(metrics) = self.metrics.as_ref() {
            metrics.cache_items.set(len as u64);
        }
        Ok(())
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        record_metrics("cache", prefix_name, "contains_key", self.metrics.as_ref())
            .call(|| Ok(self.contains_key_inner(Some(prefix_name), key)))
    }
    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
        // remove record_metrics for performance
        // record_metrics add in write_batch to reduce Instant::now system call
        let len = self.remove_inner(Some(prefix_name), key);
        if let Some(metrics) = self.metrics.as_ref() {
            metrics.cache_items.set(len as u64);
        }
        Ok(())
    }

    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        record_metrics("cache", prefix_name, "write_batch", self.metrics.as_ref()).call(|| {
            self.write_batch_inner(Some(prefix_name), batch);
            Ok(())
        })
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

    fn put_sync(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.put(prefix_name, key, value)
    }

    fn write_batch_sync(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        self.write_batch(prefix_name, batch)
    }

    fn multi_get(&self, prefix_name: &str, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>> {
        Ok(self.multi_get_inner(Some(prefix_name), keys))
    }
}

fn compose_key(prefix_name: Option<&str>, source_key: Vec<u8>) -> Vec<u8> {
    match prefix_name {
        Some(prefix_name) => {
            let temp_vec = prefix_name.as_bytes().to_vec();
            let mut compose = Vec::with_capacity(temp_vec.len() + source_key.len());
            compose.extend(temp_vec);
            compose.extend(source_key);
            compose
        }
        None => source_key,
    }
}

impl CacheStorage {
    pub fn get_inner(&self, prefix_name: Option<&str>, key: Vec<u8>) -> Option<Vec<u8>> {
        self.cache
            .lock()
            .get(&compose_key(prefix_name, key))
            .cloned()
    }

    pub fn put_inner(&self, prefix_name: Option<&str>, key: Vec<u8>, value: Vec<u8>) -> usize {
        let mut cache = self.cache.lock();
        cache.put(compose_key(prefix_name, key), value);
        cache.len()
    }

    pub fn contains_key_inner(&self, prefix_name: Option<&str>, key: Vec<u8>) -> bool {
        self.cache.lock().contains(&compose_key(prefix_name, key))
    }

    pub fn remove_inner(&self, prefix_name: Option<&str>, key: Vec<u8>) -> usize {
        let mut cache = self.cache.lock();
        cache.pop(&compose_key(prefix_name, key));
        cache.len()
    }

    pub fn write_batch_inner(&self, prefix_name: Option<&str>, batch: WriteBatch) {
        for (key, write_op) in &batch.rows {
            match write_op {
                WriteOp::Value(value) => {
                    self.put_inner(prefix_name, key.to_vec(), value.to_vec());
                }
                WriteOp::Deletion => {
                    self.remove_inner(prefix_name, key.to_vec());
                }
            };
        }
    }

    pub fn put_sync_inner(&self, prefix_name: Option<&str>, key: Vec<u8>, value: Vec<u8>) -> usize {
        self.put_inner(prefix_name, key, value)
    }

    pub fn write_batch_sync_inner(&self, prefix_name: Option<&str>, batch: WriteBatch) {
        self.write_batch_inner(prefix_name, batch)
    }

    pub fn multi_get_inner(
        &self,
        prefix_name: Option<&str>,
        keys: Vec<Vec<u8>>,
    ) -> Vec<Option<Vec<u8>>> {
        let mut cache = self.cache.lock();
        let mut result = vec![];
        for key in keys.into_iter() {
            let item = cache.get(&compose_key(prefix_name, key)).cloned();
            result.push(item);
        }
        result
    }
}
