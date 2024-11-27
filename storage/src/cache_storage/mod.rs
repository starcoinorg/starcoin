// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::{GWriteBatch, WriteBatchWithColumn};
use crate::{
    batch::WriteBatch,
    metrics::{record_metrics, StorageMetrics},
    storage::{InnerStore, WriteOp},
};
use anyhow::{Error, Result};
use core::hash::Hash;
use lru::LruCache;
use parking_lot::Mutex;
use starcoin_config::DEFAULT_CACHE_SIZE;

pub type CacheStorage = GCacheStorage<Vec<u8>, Vec<u8>>;

pub struct GCacheStorage<K: Hash + Eq + Default, V: Default> {
    cache: Mutex<LruCache<K, V>>,
    metrics: Option<StorageMetrics>,
}

impl<K: Hash + Eq + Default, V: Default> GCacheStorage<K, V> {
    pub fn new(metrics: Option<StorageMetrics>) -> Self {
        Self {
            cache: Mutex::new(LruCache::<K, V>::new(DEFAULT_CACHE_SIZE)),
            metrics,
        }
    }
    pub fn new_with_capacity(size: usize, metrics: Option<StorageMetrics>) -> Self {
        Self {
            cache: Mutex::new(LruCache::<K, V>::new(size)),
            metrics,
        }
    }
    pub fn remove_all(&self) {
        self.cache.lock().clear();
    }
}

impl<K: Hash + Eq + Default, V: Default> Default for GCacheStorage<K, V> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl InnerStore for CacheStorage {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        let composed_key = compose_key(Some(prefix_name), key);
        record_metrics("cache", prefix_name, "get", self.metrics.as_ref())
            .call(|| Ok(self.get_inner(&composed_key)))
    }

    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        // remove record_metrics for performance
        // record_metrics add in write_batch to reduce Instant::now system call
        let composed_key = compose_key(Some(prefix_name), key);
        let len = self.put_inner(composed_key, value);
        if let Some(metrics) = self.metrics.as_ref() {
            metrics.cache_items.set(len as u64);
        }
        Ok(())
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        let composed_key = compose_key(Some(prefix_name), key);
        record_metrics("cache", prefix_name, "contains_key", self.metrics.as_ref())
            .call(|| Ok(self.contains_key_inner(&composed_key)))
    }
    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
        // remove record_metrics for performance
        // record_metrics add in write_batch to reduce Instant::now system call
        let composed_key = compose_key(Some(prefix_name), key);
        let len = self.remove_inner(&composed_key);
        if let Some(metrics) = self.metrics.as_ref() {
            metrics.cache_items.set(len as u64);
        }
        Ok(())
    }

    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        let rows = batch
            .rows
            .into_iter()
            .map(|(k, v)| (compose_key(Some(prefix_name), k), v))
            .collect();
        let batch = WriteBatch { rows };
        record_metrics("cache", prefix_name, "write_batch", self.metrics.as_ref()).call(|| {
            self.write_batch_inner(batch);
            Ok(())
        })
    }

    fn write_batch_with_column(&self, batch: WriteBatchWithColumn) -> Result<()> {
        let rows = batch
            .data
            .into_iter()
            .flat_map(|data| {
                data.row_data
                    .rows
                    .iter()
                    .cloned()
                    .map(|(k, v)| (compose_key(Some(&data.column), k), v))
                    .collect::<Vec<_>>()
            })
            .collect();
        let batch = WriteBatch { rows };
        record_metrics(
            "cache",
            "write_batch_column_prefix",
            "write_batch",
            self.metrics.as_ref(),
        )
        .call(|| {
            self.write_batch_inner(batch);
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
        let composed_keys = keys
            .into_iter()
            .map(|k| compose_key(Some(prefix_name), k))
            .collect::<Vec<_>>();
        Ok(self.multi_get_inner(composed_keys.as_slice()))
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

impl<K: Hash + Eq + Default, V: Clone + Default> GCacheStorage<K, V> {
    pub fn get_inner(&self, key: &K) -> Option<V> {
        self.cache.lock().get(key).cloned()
    }

    pub fn put_inner(&self, key: K, value: V) -> usize {
        let mut cache = self.cache.lock();
        cache.put(key, value);
        cache.len()
    }

    pub fn contains_key_inner(&self, key: &K) -> bool {
        self.cache.lock().contains(key)
    }

    pub fn remove_inner(&self, key: &K) -> usize {
        let mut cache = self.cache.lock();
        cache.pop(key);
        cache.len()
    }

    pub fn write_batch_inner(&self, batch: GWriteBatch<K, V>) {
        for (key, write_op) in batch.rows {
            match write_op {
                WriteOp::Value(value) => {
                    self.put_inner(key, value);
                }
                WriteOp::Deletion => {
                    self.remove_inner(&key);
                }
            };
        }
    }

    pub fn put_sync_inner(&self, key: K, value: V) -> usize {
        self.put_inner(key, value)
    }

    pub fn write_batch_sync_inner(&self, batch: GWriteBatch<K, V>) {
        self.write_batch_inner(batch)
    }

    pub fn multi_get_inner(&self, keys: &[K]) -> Vec<Option<V>> {
        let mut cache = self.cache.lock();
        let mut result = vec![];
        for key in keys {
            let item = cache.get(key).cloned();
            result.push(item);
        }
        result
    }
}
