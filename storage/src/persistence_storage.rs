// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Repository;
use anyhow::{Error, Result};
use rocksdb::DB;

pub struct PersistenceStorage {
    db: rocksdb::DB,
}

impl PersistenceStorage {
    pub fn new() -> Self {
        PersistenceStorage {
            db: DB::open_default("path/for/rocksdb/storage").unwrap(),
        }
    }
}

impl Repository for PersistenceStorage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        unimplemented!()
    }

    fn contains_key(&self, key: Vec<u8>) -> Result<bool> {
        unimplemented!()
    }
    fn remove(&self, key: Vec<u8>) -> Result<()> {
        unimplemented!()
    }
}
