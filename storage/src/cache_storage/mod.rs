// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::{WriteBatch, WriteBatchWithColumn};
use crate::metrics::{record_metrics, StorageMetrics};
use crate::storage::{InnerStore, WriteOp};
use anyhow::{Error, Result};
use quick_cache::sync::Cache;
use starcoin_config::DEFAULT_CACHE_SIZE;
pub struct CacheStorage {
    cache: Cache<Vec<u8>, Vec<u8>>,
    metrics: Option<StorageMetrics>,
}

impl CacheStorage {
    pub fn new(metrics: Option<StorageMetrics>) -> Self {
        CacheStorage {
            cache: Cache::new(DEFAULT_CACHE_SIZE),
            metrics,
        }
    }
    pub fn new_with_capacity(size: usize, metrics: Option<StorageMetrics>) -> Self {
        CacheStorage {
            cache: Cache::new(size),
            metrics,
        }
    }

    pub fn capacity(&self) -> usize {
        self.cache.capacity() as usize
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
            .call(|| Ok(self.cache.get(&compose_key(prefix_name.to_string(), key))))
    }

    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        // remove record_metrics for performance
        // record_metrics add in write_batch to reduce Instant::now system call
        self.cache
            .insert(compose_key(prefix_name.to_string(), key), value);
        if let Some(metrics) = self.metrics.as_ref() {
            metrics.cache_items.set(self.cache.len() as u64);
        }
        Ok(())
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        record_metrics("cache", prefix_name, "contains_key", self.metrics.as_ref()).call(|| {
            Ok(self
                .cache
                .contains_key(&compose_key(prefix_name.to_string(), key)))
        })
    }
    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
        // remove record_metrics for performance
        // record_metrics add in write_batch to reduce Instant::now system call
        self.cache
            .remove(&compose_key(prefix_name.to_string(), key));
        if let Some(metrics) = self.metrics.as_ref() {
            metrics.cache_items.set(self.cache.len() as u64);
        }
        Ok(())
    }

    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        record_metrics("cache", prefix_name, "write_batch", self.metrics.as_ref()).call(|| {
            for (key, write_op) in &batch.rows {
                match write_op {
                    WriteOp::Value(value) => self.put(prefix_name, key.to_vec(), value.to_vec())?,
                    WriteOp::Deletion => self.remove(prefix_name, key.to_vec())?,
                };
            }
            Ok(())
        })
    }

    fn get_len(&self) -> Result<u64, Error> {
        Ok(self.cache.len() as u64)
    }

    // supply weak consistency after replace Mutex<LruCache> with quick_cache::sync::Cache
    fn keys(&self) -> Result<Vec<Vec<u8>>, Error> {
        let mut all_keys = vec![];
        for (key, _) in self.cache.iter() {
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
        let mut result = vec![];
        for key in keys.into_iter() {
            let item = self.cache.get(&compose_key(prefix_name.to_string(), key));
            result.push(item);
        }
        Ok(result)
    }

    fn write_batch_with_column(&self, batch: WriteBatchWithColumn) -> Result<()> {
        for data in batch.data {
            self.write_batch(&data.column, data.row_data)?;
        }
        Ok(())
    }

    fn write_batch_with_column_sync(&self, batch: WriteBatchWithColumn) -> Result<()> {
        self.write_batch_with_column(batch)
    }
}

fn compose_key(prefix_name: String, source_key: Vec<u8>) -> Vec<u8> {
    let temp_vec = prefix_name.as_bytes().to_vec();
    let mut compose = Vec::with_capacity(temp_vec.len() + source_key.len());
    compose.extend(temp_vec);
    compose.extend(source_key);
    compose
}
