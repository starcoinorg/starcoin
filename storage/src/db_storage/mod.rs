// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod iterator;

use crate::{
    batch::WriteBatch,
    storage::{InnerStore, KeyCodec, ValueCodec},
};
use anyhow::Result;
use rocksdb::ReadOptions;
use starcoin_schemadb::metrics::record_metrics;
use std::iter;
pub use {iterator::*, starcoin_schemadb::db::DBStorage};

pub trait ClassicIter {
    fn iter_with_direction<K: KeyCodec, V: ValueCodec>(
        &self,
        prefix_name: &str,
        direction: ScanDirection,
    ) -> Result<SchemaIterator<K, V>>;

    fn iter_raw<K: KeyCodec, V: ValueCodec>(
        &self,
        prefix_name: &str,
    ) -> Result<SchemaIterator<K, V>>;
    fn rev_iter_raw<K: KeyCodec, V: ValueCodec>(
        &self,
        prefix_name: &str,
    ) -> Result<SchemaIterator<K, V>>;
}

impl ClassicIter for DBStorage {
    fn iter_with_direction<K, V>(
        &self,
        prefix_name: &str,
        direction: ScanDirection,
    ) -> Result<SchemaIterator<K, V>>
    where
        K: KeyCodec,
        V: ValueCodec,
    {
        let cf_handle = self.get_cf_handle(prefix_name)?;
        Ok(SchemaIterator::new(
            self.db()
                .raw_iterator_cf_opt(cf_handle, ReadOptions::default()),
            direction,
        ))
    }
    /// Returns a forward [`SchemaIterator`] on a certain schema.
    fn iter_raw<K, V>(&self, prefix_name: &str) -> Result<SchemaIterator<K, V>>
    where
        K: KeyCodec,
        V: ValueCodec,
    {
        self.iter_with_direction(prefix_name, ScanDirection::Forward)
    }

    /// Returns a backward [`SchemaIterator`] on a certain schema.
    fn rev_iter_raw<K, V>(&self, prefix_name: &str) -> Result<SchemaIterator<K, V>>
    where
        K: KeyCodec,
        V: ValueCodec,
    {
        self.iter_with_direction(prefix_name, ScanDirection::Backward)
    }
}

impl InnerStore for DBStorage {
    fn get_raw(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        record_metrics("db", prefix_name, "get", self.metrics()).call(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            let result = self.db().get_cf(cf_handle, key.as_slice())?;
            Ok(result)
        })
    }

    fn put_raw(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        if let Some(metrics) = self.metrics() {
            metrics
                .storage_item_bytes
                .with_label_values(&[prefix_name])
                .observe((key.len() + value.len()) as f64);
        }

        record_metrics("db", prefix_name, "put", self.metrics()).call(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            self.db()
                .put_cf_opt(cf_handle, &key, &value, &Self::default_write_options())?;
            Ok(())
        })
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        record_metrics("db", prefix_name, "contains_key", self.metrics()).call(|| {
            match self.get_raw(prefix_name, key) {
                Ok(Some(_)) => Ok(true),
                _ => Ok(false),
            }
        })
    }
    fn remove_raw(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
        record_metrics("db", prefix_name, "remove", self.metrics()).call(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            self.db().delete_cf(cf_handle, &key)?;
            Ok(())
        })
    }

    /// Writes a group of records wrapped in a WriteBatch.
    fn write_batch(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        record_metrics("db", prefix_name, "write_batch", self.metrics()).call(|| {
            self.write_batch_inner(
                prefix_name,
                batch.rows.as_slice(),
                false, /*normal write*/
            )
        })
    }

    fn get_len(&self) -> Result<u64> {
        unimplemented!()
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>> {
        unimplemented!()
    }

    fn put_sync(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        if let Some(metrics) = self.metrics() {
            metrics
                .storage_item_bytes
                .with_label_values(&[prefix_name])
                .observe((key.len() + value.len()) as f64);
        }

        record_metrics("db", prefix_name, "put_sync", self.metrics()).call(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            self.db()
                .put_cf_opt(cf_handle, &key, &value, &Self::sync_write_options())?;
            Ok(())
        })
    }

    fn write_batch_sync(&self, prefix_name: &str, batch: WriteBatch) -> Result<()> {
        record_metrics("db", prefix_name, "write_batch_sync", self.metrics())
            .call(|| self.write_batch_inner(prefix_name, batch.rows.as_slice(), true))
    }

    fn multi_get(&self, prefix_name: &str, keys: Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>> {
        record_metrics("db", prefix_name, "multi_get", self.metrics()).call(|| {
            let cf_handle = self.get_cf_handle(prefix_name)?;
            let cf_handles = iter::repeat(&cf_handle)
                .take(keys.len())
                .collect::<Vec<_>>();
            let keys_multi = keys
                .iter()
                .zip(cf_handles)
                .map(|(key, handle)| (handle, key.as_slice()))
                .collect::<Vec<_>>();

            let result = self.db().multi_get_cf(keys_multi);
            let mut res = vec![];
            for item in result {
                let item = item?;
                res.push(item);
            }
            Ok(res)
        })
    }
}
