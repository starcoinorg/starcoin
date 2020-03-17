// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Repository;
use anyhow::{Error, Result};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct MemoryStorage {
    map: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            map: RwLock::new(HashMap::new()),
        }
    }
}

impl Repository for MemoryStorage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.map.read().unwrap().get(key).map(|v| v.to_vec()))
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.map.write().unwrap().insert(key, value);
        Ok(())
    }

    fn contains_key(&self, key: Vec<u8>) -> Result<bool> {
        Ok(self.map.read().unwrap().contains_key(&key))
    }
    fn remove(&self, key: Vec<u8>) -> Result<()> {
        self.map.write().unwrap().remove(&key);
        Ok(())
    }

    fn get_len(&self) -> Result<u64, Error> {
        Ok(self.map.read().unwrap().len() as u64)
    }

    fn keys(&self) -> Result<Vec<Vec<u8>>, Error> {
        let mut all_keys = vec![];
        for key in self.map.read().unwrap().keys() {
            all_keys.push(key.to_vec());
        }
        Ok(all_keys)
    }
}
