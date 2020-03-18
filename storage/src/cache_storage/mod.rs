// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::InnerRepository;
use anyhow::{Error, Result};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct CacheStorage {
    map: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
}

impl CacheStorage {
    pub fn new() -> Self {
        CacheStorage {
            map: RwLock::new(HashMap::new()),
        }
    }
}

impl InnerRepository for CacheStorage {
    fn get(&self, prefix_name: &str, key: Vec<u8>) -> Result<Option<Vec<u8>>> {
        let compose = compose_key(prefix_name.to_string(), key)?;
        Ok(self
            .map
            .read()
            .unwrap()
            .get(compose.as_slice())
            .map(|v| v.to_vec()))
    }

    fn put(&self, prefix_name: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.map
            .write()
            .unwrap()
            .insert(compose_key(prefix_name.to_string(), key)?, value);
        Ok(())
    }

    fn contains_key(&self, prefix_name: &str, key: Vec<u8>) -> Result<bool> {
        let compose = compose_key(prefix_name.to_string(), key)?;
        Ok(self.map.read().unwrap().contains_key(compose.as_slice()))
    }
    fn remove(&self, prefix_name: &str, key: Vec<u8>) -> Result<()> {
        let compose = compose_key(prefix_name.to_string(), key)?;
        self.map.write().unwrap().remove(compose.as_slice());
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

fn compose_key(prefix_name: String, source_key: Vec<u8>) -> Result<Vec<u8>> {
    let temp_vec = prefix_name.as_bytes().to_vec();
    let mut compose = Vec::with_capacity(temp_vec.len() + source_key.len());
    compose.extend(temp_vec);
    compose.extend(source_key);
    Ok(compose)
}
