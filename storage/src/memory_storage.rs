// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::Repository;
use anyhow::{Error, Result};
use std::cell::RefCell;
use std::collections::HashMap;

pub struct MemoryStorage {
    map: RefCell<HashMap<Vec<u8>, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            map: RefCell::new(HashMap::new()),
        }
    }
}

impl Repository for MemoryStorage {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.map.borrow().get(key).map(|v| v.to_vec()))
    }

    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.map.borrow_mut().insert(key, value);
        Ok(())
    }

    fn contains_key(&self, key: Vec<u8>) -> Result<bool> {
        Ok(self.map.borrow_mut().contains_key(&key))
    }
    fn remove(&self, key: Vec<u8>) -> Result<()> {
        self.map.borrow_mut().remove(&key);
        Ok(())
    }

    fn get_len(&self) -> Result<u64, Error> {
        Ok(self.map.borrow_mut().len() as u64)
    }
}
